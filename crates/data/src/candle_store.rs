//! High-performance flat candle store — one file per (symbol, interval).
//!
//! Stores candles as contiguous fixed-size records for zero-overhead bulk
//! reads. A single 24MB file replaces 300+ individual cache files.
//!
//! Layout v3: `[u32 magic][u64 count][CandleRecord × count][u32 crc32]`
//! Each CandleRecord is 64 bytes. The trailing CRC32 covers everything
//! before it (magic + count + records). Magic = 0x43444C33 ("CDL3")
//! identifies v3 files. Files without this magic are discarded and re-fetched.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use claude_trader_models::{Candle, FundingRate};

use crate::coverage::Coverage;

const FILE_MAGIC: u32 = 0x43444C33; // "CDL3"
const FUND_MAGIC: u32 = 0x464E4432; // "FND2"
const HEADER_SIZE: usize = 4 + 8; // magic(4) + count(8)
const FOOTER_SIZE: usize = 4; // trailing CRC32

/// 64-byte fixed-size candle record.
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct CandleRecord {
    open_time_ms: i64,
    close_time_ms: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    taker_buy_volume: f64,
}

const RECORD_SIZE: usize = std::mem::size_of::<CandleRecord>(); // 64 bytes

/// CRC32 (ISO 3309 / ITU-T V.42) — used for file-level integrity checks.
/// Delegates to `crc32fast`, which uses PCLMULQDQ on x86-64 and a table
/// fallback elsewhere. Output is bit-identical to the prior bitwise loop,
/// so every existing `.bin` file on disk stays readable.
fn crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

impl CandleRecord {
    fn from_candle(c: &Candle) -> Self {
        Self {
            open_time_ms: c.open_time.timestamp_millis(),
            close_time_ms: c.close_time.timestamp_millis(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
            taker_buy_volume: c.taker_buy_volume,
        }
    }

    /// Write the record as explicit little-endian field bytes. Replaces the
    /// previous `transmute` of a `#[repr(C, packed)]` struct — still LE-only
    /// on disk, but no `unsafe` on the write path and resilient to future
    /// struct-layout refactors.
    fn write_le(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.open_time_ms.to_le_bytes());
        out.extend_from_slice(&self.close_time_ms.to_le_bytes());
        out.extend_from_slice(&self.open.to_le_bytes());
        out.extend_from_slice(&self.high.to_le_bytes());
        out.extend_from_slice(&self.low.to_le_bytes());
        out.extend_from_slice(&self.close.to_le_bytes());
        out.extend_from_slice(&self.volume.to_le_bytes());
        out.extend_from_slice(&self.taker_buy_volume.to_le_bytes());
    }

    fn to_candle(&self) -> Candle {
        Candle {
            open_time: Utc
                .timestamp_millis_opt(self.open_time_ms)
                .single()
                .unwrap_or_default(),
            close_time: Utc
                .timestamp_millis_opt(self.close_time_ms)
                .single()
                .unwrap_or_default(),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            taker_buy_volume: self.taker_buy_volume,
        }
    }
}

// ---------------------------------------------------------------------------
// FundingRecord — compact 24-byte fixed-size record for funding rates.
// ---------------------------------------------------------------------------

const FUND_RECORD_SIZE: usize = std::mem::size_of::<FundingRecord>(); // 24 bytes

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct FundingRecord {
    timestamp_ms: i64,
    funding_rate: f64,
    mark_price: f64, // NaN when mark_price is None (0.0 is a valid price)
}

impl FundingRecord {
    fn from_funding_rate(fr: &FundingRate) -> Self {
        Self {
            timestamp_ms: fr.timestamp.timestamp_millis(),
            funding_rate: fr.funding_rate,
            mark_price: fr.mark_price.unwrap_or(f64::NAN),
        }
    }

    /// Write the record as explicit little-endian field bytes. See
    /// `CandleRecord::write_le` for rationale.
    fn write_le(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.timestamp_ms.to_le_bytes());
        out.extend_from_slice(&self.funding_rate.to_le_bytes());
        out.extend_from_slice(&self.mark_price.to_le_bytes());
    }

    fn to_funding_rate(&self) -> FundingRate {
        FundingRate {
            timestamp: Utc
                .timestamp_millis_opt(self.timestamp_ms)
                .single()
                .unwrap_or_default(),
            funding_rate: self.funding_rate,
            mark_price: if self.mark_price.is_nan() {
                None
            } else {
                Some(self.mark_price)
            },
        }
    }
}

/// Validate file-level CRC. Returns the content slice (everything before
/// the footer) on success. On failure, deletes the corrupt file and
/// returns None.
fn validate_and_strip_footer<'a>(bytes: &'a [u8], path: &std::path::Path) -> Option<&'a [u8]> {
    if bytes.len() < HEADER_SIZE + FOOTER_SIZE {
        return None;
    }
    let content = &bytes[..bytes.len() - FOOTER_SIZE];
    let stored = u32::from_le_bytes(bytes[bytes.len() - FOOTER_SIZE..].try_into().unwrap());
    if crc32(content) != stored {
        eprintln!(
            "WARNING: {} failed CRC check — deleting corrupt file.",
            path.display()
        );
        fs::remove_file(path).ok();
        return None;
    }
    Some(content)
}

/// Raw byte cache entry for 1m files. File-level CRC is verified on load;
/// range queries operate directly on the validated byte buffer.
struct ValidatedCache {
    data: Arc<Vec<u8>>,
    count: usize,
}

/// High-performance candle store with in-memory caching.
pub struct CandleStore {
    store_dir: PathBuf,
    /// In-memory cache: (symbol, interval) → sorted candles (for 1h etc.)
    mem: HashMap<(String, String), Vec<Candle>>,
    /// Raw byte cache for 1m files. File-level CRC verified on load.
    raw_cache: HashMap<(String, String), ValidatedCache>,
    /// In-memory cache for funding rates: symbol → sorted funding rates.
    funding_mem: HashMap<String, Vec<FundingRate>>,
    /// Tracks which millisecond ranges have been probed against the upstream
    /// API for each `(symbol, interval)` candle file. Separate from the data
    /// itself: an empty fetch result still counts as a probe, so repeat
    /// fetches of known-empty trailing ranges are suppressed. See
    /// [`crate::coverage::Coverage`] for the invariants and file format.
    candle_coverage: HashMap<(String, String), Coverage>,
    /// Per-symbol probed-range tracking for funding files. Same semantics
    /// as `candle_coverage`.
    funding_coverage: HashMap<String, Coverage>,
}

impl CandleStore {
    pub fn new() -> Self {
        let store_dir = home_dir().join(".claude_trader").join("candle_store");
        fs::create_dir_all(&store_dir).ok();
        Self {
            store_dir,
            mem: HashMap::new(),
            raw_cache: HashMap::new(),
            funding_mem: HashMap::new(),
            candle_coverage: HashMap::new(),
            funding_coverage: HashMap::new(),
        }
    }

    /// Get candles for a specific time range.
    ///
    /// For 1h (and other non-minute) intervals, loads the full file into memory
    /// on first access and filters from the cache.
    /// For 1m intervals, reads directly from disk with range filtering to avoid
    /// holding gigabytes of minute data in RAM.
    pub fn get_range(
        &mut self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Candle> {
        if interval == "1m" {
            // Disk-direct path: read only the needed range, don't cache in mem
            return self.load_range_from_file(symbol, interval, start, end);
        }

        self.get_range_ref(symbol, interval, start, end).to_vec()
    }

    /// Borrowed-slice variant of `get_range` for cached (non-1m) intervals.
    ///
    /// Returns `&[Candle]` directly from the in-memory cache — zero allocation.
    /// Panics if called with `"1m"` (use `get_range` for disk-direct minute data).
    pub fn get_range_ref(
        &mut self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> &[Candle] {
        assert_ne!(
            interval, "1m",
            "get_range_ref not supported for 1m intervals"
        );

        let key = (symbol.to_string(), interval.to_string());
        if !self.mem.contains_key(&key) {
            let candles = self.load_from_file(symbol, interval);
            self.mem.insert(key.clone(), candles);
        }

        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();

        match self.mem.get(&key) {
            Some(candles) => {
                // Binary search for start position
                let lo = candles.partition_point(|c| c.close_time.timestamp_millis() < start_ms);
                // Binary search for end position
                let hi = candles.partition_point(|c| c.open_time.timestamp_millis() < end_ms);
                &candles[lo..hi]
            }
            None => &[],
        }
    }

    /// Return the full cached slice for a non-1m interval — zero allocation,
    /// zero range filtering.  Ensures the file is loaded on first access.
    ///
    /// Use this when the caller will do its own binary-search narrowing and
    /// does not want the overhead of two extra partition_points per call.
    pub fn get_all_ref(&mut self, symbol: &str, interval: &str) -> &[Candle] {
        assert_ne!(interval, "1m", "get_all_ref not supported for 1m intervals");

        let key = (symbol.to_string(), interval.to_string());
        if !self.mem.contains_key(&key) {
            let candles = self.load_from_file(symbol, interval);
            self.mem.insert(key.clone(), candles);
        }

        self.mem.get(&key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Store candles (merges with existing data, deduplicates, sorts).
    ///
    /// For 1m intervals, writes to disk only (no memory cache) to avoid OOM.
    pub fn put(&mut self, symbol: &str, interval: &str, new_candles: &[Candle]) {
        if new_candles.is_empty() {
            return;
        }

        let key = (symbol.to_string(), interval.to_string());

        // Take ownership from mem cache or load from disk (avoids cloning)
        let mut all = if let Some(existing) = self.mem.remove(&key) {
            existing
        } else {
            self.load_from_file(symbol, interval)
        };

        // Merge
        all.extend_from_slice(new_candles);

        // Deduplicate by open_time_ms — keep the LAST occurrence (newest data wins).
        // sort_by_key is stable, so among duplicates the later (new) entry comes last.
        // Reverse, dedup (keeps first = newest), then reverse back.
        all.sort_by_key(|c| c.open_time.timestamp_millis());
        all.reverse();
        all.dedup_by_key(|c| c.open_time.timestamp_millis());
        all.reverse();

        // Write to file
        self.write_to_file(symbol, interval, &all);

        if interval == "1m" {
            // Invalidate raw byte cache so next read picks up new data
            self.raw_cache.remove(&key);
        } else {
            // Cache deserialized candles for fast access
            self.mem.insert(key, all);
        }
    }

    /// Number of candles loaded in memory.
    pub fn mem_candle_count(&self) -> usize {
        self.mem.values().map(|v| v.len()).sum()
    }

    /// Pre-load 1m files into raw_cache in parallel (bounded to 4 concurrent
    /// reads). Symbols already cached are skipped. Call this before a batch of
    /// `get_range(sym, "1m", ...)` to avoid sequential cold loads.
    pub fn preload_1m(&mut self, symbols: &[String]) {
        let needed: Vec<(String, PathBuf)> = symbols
            .iter()
            .filter(|sym| {
                let key = (sym.to_string(), "1m".to_string());
                !self.raw_cache.contains_key(&key)
            })
            .map(|sym| (sym.clone(), self.file_path(sym, "1m")))
            .filter(|(_, path)| path.exists())
            .collect();

        if needed.is_empty() {
            return;
        }

        // Bounded concurrency: 4 files at a time via chunks + scoped threads.
        let mut results: Vec<(String, ValidatedCache)> = Vec::new();
        for chunk in needed.chunks(4) {
            std::thread::scope(|s| {
                let handles: Vec<_> = chunk
                    .iter()
                    .map(|(sym, path)| {
                        let sym = sym.clone();
                        let path = path.clone();
                        s.spawn(move || -> Option<(String, ValidatedCache)> {
                            load_1m_raw(&path).map(|vc| (sym, vc))
                        })
                    })
                    .collect();
                for h in handles {
                    if let Some(entry) = h.join().unwrap() {
                        results.push(entry);
                    }
                }
            });
        }

        // Worker threads can't hold `&mut self`, so they can only delete the
        // `.bin` on corruption (via `load_1m_raw`'s built-in handling). The
        // `.cov` sidecar and any cached coverage entries still need to go —
        // otherwise a later `candle_coverage_gaps` call would trust a
        // stale sidecar describing data that no longer exists. Reconcile
        // attempted vs. successful loads here and invalidate the
        // difference on the main thread.
        let mut successful: std::collections::HashSet<String> =
            std::collections::HashSet::with_capacity(results.len());
        for (sym, vc) in results {
            successful.insert(sym.clone());
            self.raw_cache.insert((sym, "1m".to_string()), vc);
        }
        for (sym, _) in &needed {
            if !successful.contains(sym) {
                self.invalidate_candle(sym, "1m");
            }
        }
    }

    // -------------------------------------------------------------------
    // File I/O
    // -------------------------------------------------------------------

    fn file_path(&self, symbol: &str, interval: &str) -> PathBuf {
        self.store_dir.join(format!("{symbol}_{interval}.bin"))
    }

    fn coverage_path_candle(&self, symbol: &str, interval: &str) -> PathBuf {
        self.store_dir.join(format!("{symbol}_{interval}.cov"))
    }

    fn coverage_path_funding(&self, symbol: &str) -> PathBuf {
        self.store_dir.join(format!("{symbol}_funding.cov"))
    }

    /// Drop all traces of a candle `(symbol, interval)` from the store:
    /// in-memory candle cache, raw-byte cache, in-memory coverage,
    /// on-disk `.bin`, and on-disk `.cov` sidecar. Idempotent — missing
    /// files are silently ignored.
    ///
    /// This is the *only* sanctioned way to mark a candle file as corrupt.
    /// Clearing just the `.bin` would leave a stale `.cov` that would lie
    /// about what has been probed, causing coverage-driven gap detection
    /// to suppress the re-fetch that corruption triggers.
    pub fn invalidate_candle(&mut self, symbol: &str, interval: &str) {
        let key = (symbol.to_string(), interval.to_string());
        self.mem.remove(&key);
        self.raw_cache.remove(&key);
        self.candle_coverage.remove(&key);
        let _ = fs::remove_file(self.file_path(symbol, interval));
        let _ = fs::remove_file(self.coverage_path_candle(symbol, interval));
    }

    /// Funding analog of [`invalidate_candle`].
    pub fn invalidate_funding(&mut self, symbol: &str) {
        self.funding_mem.remove(symbol);
        self.funding_coverage.remove(symbol);
        let _ = fs::remove_file(self.funding_path(symbol));
        let _ = fs::remove_file(self.coverage_path_funding(symbol));
    }

    /// Return coverage for `(symbol, interval)`, loading or synthesizing
    /// it on first access. Inserts the result into `self.candle_coverage`
    /// so subsequent calls are O(1).
    fn get_candle_coverage(&mut self, symbol: &str, interval: &str) -> &Coverage {
        let key = (symbol.to_string(), interval.to_string());
        if !self.candle_coverage.contains_key(&key) {
            let cov = self.load_or_synthesize_candle_coverage(symbol, interval);
            self.candle_coverage.insert(key.clone(), cov);
        }
        self.candle_coverage.get(&key).expect("just inserted")
    }

    /// Return coverage for `symbol` funding, loading or synthesizing on
    /// first access.
    fn get_funding_coverage(&mut self, symbol: &str) -> &Coverage {
        if !self.funding_coverage.contains_key(symbol) {
            let cov = self.load_or_synthesize_funding_coverage(symbol);
            self.funding_coverage.insert(symbol.to_string(), cov);
        }
        self.funding_coverage
            .get(symbol)
            .expect("just inserted")
    }

    /// Run the normal data-load path to validate the `.bin` against
    /// CRC, magic, and truncation checks. Failures route through
    /// `invalidate_candle`, which deletes the `.cov` sidecar in
    /// lock-step — so after this returns, any remaining `.cov` is
    /// guaranteed to describe the current `.bin` (or neither exists).
    ///
    /// Cheap when caches are already warm: a single `HashMap::contains_key`
    /// check on the hot path. Only runs the expensive CRC scan on first
    /// access per `(symbol, interval)` per store lifetime.
    fn ensure_candle_data_validated(&mut self, symbol: &str, interval: &str) {
        let key = (symbol.to_string(), interval.to_string());
        if interval == "1m" {
            if self.raw_cache.contains_key(&key) {
                return;
            }
            let path = self.file_path(symbol, "1m");
            if !path.exists() {
                return;
            }
            match load_1m_raw(&path) {
                Some(vc) => {
                    self.raw_cache.insert(key, vc);
                }
                None => {
                    // Corruption — `.bin` already removed by
                    // `load_1m_raw`'s own deletions; invalidate to
                    // drop the orphan `.cov` and caches.
                    self.invalidate_candle(symbol, "1m");
                }
            }
        } else {
            if self.mem.contains_key(&key) {
                return;
            }
            let path = self.file_path(symbol, interval);
            if !path.exists() {
                return;
            }
            // `load_from_file` handles all three corruption classes
            // (CRC, magic, truncation) internally via
            // `invalidate_candle`. Cache the result so subsequent
            // reads don't re-validate.
            let candles = self.load_from_file(symbol, interval);
            self.mem.insert(key, candles);
        }
    }

    /// Funding analog of `ensure_candle_data_validated`.
    fn ensure_funding_data_validated(&mut self, symbol: &str) {
        if self.funding_mem.contains_key(symbol) {
            return;
        }
        let path = self.funding_path(symbol);
        if !path.exists() {
            return;
        }
        let rates = self.load_funding_from_file(symbol);
        self.funding_mem.insert(symbol.to_string(), rates);
    }

    fn load_or_synthesize_candle_coverage(&mut self, symbol: &str, interval: &str) -> Coverage {
        // Validate the `.bin` FIRST. If it's corrupt, the load path
        // invalidates (removes `.cov` + caches) before we consult the
        // sidecar. This ordering is load-bearing: without it, a valid
        // sidecar over a corrupt `.bin` would report "no gaps",
        // suppress refetch, and leak empty data to the caller.
        self.ensure_candle_data_validated(symbol, interval);

        let cov_path = self.coverage_path_candle(symbol, interval);

        // At this point any surviving `.cov` reflects the current
        // `.bin` state (or neither exists). Coverage may legitimately
        // exist without a `.bin` — empty probes, pre-listing ranges,
        // delisted symbols — so a valid sidecar is trusted unconditionally.
        if let Some(cov) = Coverage::load_from_file(&cov_path) {
            return cov;
        }

        // No sidecar yet. If the `.bin` doesn't exist either, there's
        // nothing to synthesize from — return empty and skip persistence.
        // If the `.bin` does exist, auto-heal by scanning it for
        // contiguous runs, splitting on gaps wider than a tolerance.
        if !self.file_path(symbol, interval).exists() {
            return Coverage::new();
        }
        let cov = self.synthesize_candle_coverage(symbol, interval);
        if !cov.is_empty() {
            let _ = cov.write_to_file(&cov_path);
        }
        cov
    }

    fn load_or_synthesize_funding_coverage(&mut self, symbol: &str) -> Coverage {
        // Validate before trusting the sidecar — same ordering
        // invariant as `load_or_synthesize_candle_coverage`.
        self.ensure_funding_data_validated(symbol);

        let cov_path = self.coverage_path_funding(symbol);
        if let Some(cov) = Coverage::load_from_file(&cov_path) {
            return cov;
        }

        if !self.funding_path(symbol).exists() {
            return Coverage::new();
        }
        // Reuse the just-validated cached rates if present; otherwise
        // do a fresh load (this branch is only reached when the cache
        // is empty, e.g. the `.bin` was just created).
        let rates = match self.funding_mem.get(symbol) {
            Some(r) => r.clone(),
            None => self.load_funding_from_file(symbol),
        };
        let cov = synthesize_coverage_from_funding(&rates);
        if !cov.is_empty() {
            let _ = cov.write_to_file(&cov_path);
        }
        cov
    }

    /// Scan on-disk data for the symbol/interval pair and build a
    /// Coverage that splits at gaps exceeding `interval_ms * 1.5`. For
    /// 1m this reads the raw bytes directly to avoid deserializing
    /// hundreds of MB; other intervals use the deserialized candles.
    ///
    /// Relies on `ensure_candle_data_validated` having populated the
    /// appropriate cache first — any file corruption has already been
    /// invalidated by this point, so reading an empty cache here means
    /// "no data to synthesize from," not "unvalidated data."
    fn synthesize_candle_coverage(&mut self, symbol: &str, interval: &str) -> Coverage {
        let key = (symbol.to_string(), interval.to_string());
        if interval == "1m" {
            match self.raw_cache.get(&key) {
                Some(cache) => synthesize_coverage_from_1m_raw(&cache.data, cache.count),
                None => Coverage::new(),
            }
        } else {
            match self.mem.get(&key) {
                Some(candles) => synthesize_coverage_from_candles(candles),
                None => Coverage::new(),
            }
        }
    }

    fn load_from_file(&mut self, symbol: &str, interval: &str) -> Vec<Candle> {
        let path = self.file_path(symbol, interval);
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };

        let content = match validate_and_strip_footer(&bytes, &path) {
            Some(c) => c,
            None => {
                // CRC failure — validate_and_strip_footer deleted the
                // `.bin` already, but the `.cov` sidecar and in-memory
                // caches still need clearing.
                self.invalidate_candle(symbol, interval);
                return Vec::new();
            }
        };

        let magic = u32::from_le_bytes(content[0..4].try_into().unwrap());
        if magic != FILE_MAGIC {
            eprintln!(
                "WARNING: {symbol}_{interval}.bin has wrong magic — discarding."
            );
            self.invalidate_candle(symbol, interval);
            return Vec::new();
        }

        let count = u64::from_le_bytes(content[4..12].try_into().unwrap()) as usize;
        let data_bytes = &content[HEADER_SIZE..];
        let available = data_bytes.len() / RECORD_SIZE;

        if available < count {
            eprintln!(
                "WARNING: {symbol}_{interval}.bin truncated — header expects {count} records, payload holds {available}. Discarding."
            );
            self.invalidate_candle(symbol, interval);
            return Vec::new();
        }

        if count == 0 {
            return Vec::new();
        }

        let mut candles = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * RECORD_SIZE;
            debug_assert!(
                offset + RECORD_SIZE <= data_bytes.len(),
                "load_from_file: record {i} out of bounds (offset={offset}, data_len={})",
                data_bytes.len()
            );
            let record = unsafe {
                std::ptr::read_unaligned(data_bytes[offset..].as_ptr() as *const CandleRecord)
            };
            candles.push(record.to_candle());
        }

        candles
    }

    /// Read only candles within [start, end) for 1m data.
    ///
    /// On first access, reads the file into the raw byte cache.
    /// Subsequent calls do binary search + deserialize only.
    fn load_range_from_file(
        &mut self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Candle> {
        let key = (symbol.to_string(), interval.to_string());

        if !self.raw_cache.contains_key(&key) {
            let path = self.file_path(symbol, interval);
            if let Some(vc) = load_1m_raw(&path) {
                self.raw_cache.insert(key.clone(), vc);
            } else {
                // `load_1m_raw` returns `None` for both "file absent" and
                // "file corrupt" (it already deletes the `.bin` in the
                // corrupt case). Either way, any lingering `.cov` is
                // stale and in-memory caches must be cleared.
                self.invalidate_candle(symbol, interval);
                return Vec::new();
            }
        }

        let cache = match self.raw_cache.get(&key) {
            Some(c) => c,
            None => return Vec::new(),
        };

        if cache.count == 0 {
            return Vec::new();
        }

        let data = &cache.data;
        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();

        let lo = binary_search_records(data, cache.count, start_ms, true);
        let hi = binary_search_records(data, cache.count, end_ms, false);

        let mut candles = Vec::with_capacity(hi.saturating_sub(lo));
        for i in lo..hi {
            let offset = i * RECORD_SIZE;
            debug_assert!(
                offset + RECORD_SIZE <= data.len(),
                "load_range_from_file: record {i} out of bounds (offset={offset}, data_len={})",
                data.len()
            );
            let record =
                unsafe { std::ptr::read_unaligned(data[offset..].as_ptr() as *const CandleRecord) };
            candles.push(record.to_candle());
        }
        candles
    }

    fn write_to_file(&self, symbol: &str, interval: &str, candles: &[Candle]) {
        let path = self.file_path(symbol, interval);
        let tmp = path.with_extension("tmp");

        let count = candles.len() as u64;
        let mut buf = Vec::with_capacity(HEADER_SIZE + candles.len() * RECORD_SIZE + FOOTER_SIZE);
        buf.extend_from_slice(&FILE_MAGIC.to_le_bytes());
        buf.extend_from_slice(&count.to_le_bytes());

        for c in candles {
            CandleRecord::from_candle(c).write_le(&mut buf);
        }

        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());

        let ok = (|| -> std::io::Result<()> {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&buf)?;
            f.sync_all()?;
            Ok(())
        })();

        if ok.is_ok() {
            fs::rename(&tmp, &path).ok();
        } else {
            fs::remove_file(&tmp).ok();
        }
    }

    // -------------------------------------------------------------------
    // Funding rate storage (compact 24-byte records, FND2 magic)
    // -------------------------------------------------------------------

    fn funding_path(&self, symbol: &str) -> PathBuf {
        self.store_dir.join(format!("{symbol}_funding.bin"))
    }

    /// Get funding rates for a time range.
    pub fn get_funding_range(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<FundingRate> {
        if !self.funding_mem.contains_key(symbol) {
            let rates = self.load_funding_from_file(symbol);
            self.funding_mem.insert(symbol.to_string(), rates);
        }

        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();

        match self.funding_mem.get(symbol) {
            Some(rates) => {
                let lo = rates.partition_point(|fr| fr.timestamp.timestamp_millis() < start_ms);
                let hi = rates.partition_point(|fr| fr.timestamp.timestamp_millis() < end_ms);
                rates[lo..hi].to_vec()
            }
            None => Vec::new(),
        }
    }

    /// Store funding rates (merges with existing, deduplicates, sorts).
    pub fn put_funding(&mut self, symbol: &str, new_rates: &[FundingRate]) {
        if new_rates.is_empty() {
            return;
        }

        let mut all = if let Some(existing) = self.funding_mem.remove(symbol) {
            existing
        } else {
            self.load_funding_from_file(symbol)
        };

        all.extend_from_slice(new_rates);
        // Keep the LAST occurrence (newest data wins) — same pattern as put().
        all.sort_by_key(|fr| fr.timestamp.timestamp_millis());
        all.reverse();
        all.dedup_by_key(|fr| fr.timestamp.timestamp_millis());
        all.reverse();

        self.write_funding_to_file(symbol, &all);
        self.funding_mem.insert(symbol.to_string(), all);
    }

    fn load_funding_from_file(&mut self, symbol: &str) -> Vec<FundingRate> {
        let path = self.funding_path(symbol);
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };

        let content = match validate_and_strip_footer(&bytes, &path) {
            Some(c) => c,
            None => {
                self.invalidate_funding(symbol);
                return Vec::new();
            }
        };

        let magic = u32::from_le_bytes(content[0..4].try_into().unwrap());
        if magic != FUND_MAGIC {
            eprintln!("WARNING: {symbol}_funding.bin has wrong magic — discarding.");
            self.invalidate_funding(symbol);
            return Vec::new();
        }

        let count = u64::from_le_bytes(content[4..12].try_into().unwrap()) as usize;
        let data_bytes = &content[HEADER_SIZE..];
        let available = data_bytes.len() / FUND_RECORD_SIZE;

        if available < count {
            eprintln!(
                "WARNING: {symbol}_funding.bin truncated — header expects {count} records, payload holds {available}. Discarding."
            );
            self.invalidate_funding(symbol);
            return Vec::new();
        }

        if count == 0 {
            return Vec::new();
        }

        let mut rates = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * FUND_RECORD_SIZE;
            debug_assert!(
                offset + FUND_RECORD_SIZE <= data_bytes.len(),
                "load_funding_from_file: record {i} out of bounds (offset={offset}, data_len={})",
                data_bytes.len()
            );
            let record = unsafe {
                std::ptr::read_unaligned(data_bytes[offset..].as_ptr() as *const FundingRecord)
            };
            rates.push(record.to_funding_rate());
        }

        rates
    }

    fn write_funding_to_file(&self, symbol: &str, rates: &[FundingRate]) {
        let path = self.funding_path(symbol);
        let tmp = path.with_extension("tmp");

        let count = rates.len() as u64;
        let mut buf =
            Vec::with_capacity(HEADER_SIZE + rates.len() * FUND_RECORD_SIZE + FOOTER_SIZE);
        buf.extend_from_slice(&FUND_MAGIC.to_le_bytes());
        buf.extend_from_slice(&count.to_le_bytes());

        for fr in rates {
            FundingRecord::from_funding_rate(fr).write_le(&mut buf);
        }

        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());

        let ok = (|| -> std::io::Result<()> {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&buf)?;
            f.sync_all()?;
            Ok(())
        })();

        if ok.is_ok() {
            fs::rename(&tmp, &path).ok();
        } else {
            fs::remove_file(&tmp).ok();
        }
    }

    // -------------------------------------------------------------------
    // Public coverage API
    // -------------------------------------------------------------------

    /// Ranges within `[start, end)` that have not yet been probed against
    /// Binance for the given candle `(symbol, interval)`.
    ///
    /// Reflects the actual probed set — non-contiguous backfill is detected
    /// correctly, and the trailing tail is treated as uncovered until
    /// explicitly recorded via [`record_candle_coverage`].
    pub fn candle_coverage_gaps(
        &mut self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();
        let cov = self.get_candle_coverage(symbol, interval);
        cov.gaps(start_ms, end_ms)
            .into_iter()
            .filter_map(|(s, e)| {
                let ds = DateTime::<Utc>::from_timestamp_millis(s)?;
                let de = DateTime::<Utc>::from_timestamp_millis(e)?;
                Some((ds, de))
            })
            .collect()
    }

    /// Record `[start, end)` as having been probed for the given candle
    /// file. The range is coalesced into existing coverage and the
    /// sidecar is rewritten atomically.
    ///
    /// Callers should invoke this even when the probe returned no rows —
    /// the whole point of coverage is to stop re-probing known-empty
    /// trailing ranges.
    pub fn record_candle_coverage(
        &mut self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) {
        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();
        if start_ms >= end_ms {
            return;
        }
        // Force load-or-synthesize before mutating.
        let _ = self.get_candle_coverage(symbol, interval);
        let path = self.coverage_path_candle(symbol, interval);
        let key = (symbol.to_string(), interval.to_string());
        let cov = self
            .candle_coverage
            .get_mut(&key)
            .expect("coverage loaded above");
        cov.add(start_ms, end_ms);
        if let Err(e) = cov.write_to_file(&path) {
            eprintln!(
                "WARNING: failed to persist coverage for {symbol}_{interval}.cov: {e}"
            );
        }
    }

    /// Funding analog of [`candle_coverage_gaps`].
    pub fn funding_coverage_gaps(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();
        let cov = self.get_funding_coverage(symbol);
        cov.gaps(start_ms, end_ms)
            .into_iter()
            .filter_map(|(s, e)| {
                let ds = DateTime::<Utc>::from_timestamp_millis(s)?;
                let de = DateTime::<Utc>::from_timestamp_millis(e)?;
                Some((ds, de))
            })
            .collect()
    }

    /// Funding analog of [`record_candle_coverage`].
    pub fn record_funding_coverage(
        &mut self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) {
        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();
        if start_ms >= end_ms {
            return;
        }
        let _ = self.get_funding_coverage(symbol);
        let path = self.coverage_path_funding(symbol);
        let cov = self
            .funding_coverage
            .get_mut(symbol)
            .expect("coverage loaded above");
        cov.add(start_ms, end_ms);
        if let Err(e) = cov.write_to_file(&path) {
            eprintln!(
                "WARNING: failed to persist coverage for {symbol}_funding.cov: {e}"
            );
        }
    }
}

// -----------------------------------------------------------------------
// Coverage synthesis (auto-heal for existing stores without a .cov sidecar)
// -----------------------------------------------------------------------

/// Gap threshold used when scanning existing candle data: any adjacency
/// larger than `interval_ms * 3 / 2` (1.5×) splits the coverage into two
/// intervals. Tolerates up to one missing candle as normal jitter, flags
/// wider gaps as likely local backfill discontinuities.
fn candle_split_threshold_ms(interval_ms: i64) -> i64 {
    interval_ms.saturating_mul(3) / 2
}

/// Derive the nominal interval from data by taking the *minimum* of all
/// consecutive open-time differences. Using the first pair is unsafe —
/// if those two candles straddle a real hole, the inferred interval is
/// inflated and the split threshold becomes large enough to swallow
/// genuine interior gaps. The minimum difference reflects the true
/// cadence; any wider adjacency is a gap we want to flag.
fn effective_interval_ms(candles: &[Candle]) -> i64 {
    if candles.len() < 2 {
        let a = candles[0].open_time.timestamp_millis();
        let b = candles[0].close_time.timestamp_millis();
        return (b - a).max(1);
    }
    let mut min_diff = i64::MAX;
    for pair in candles.windows(2) {
        let diff = pair[1].open_time.timestamp_millis() - pair[0].open_time.timestamp_millis();
        if diff > 0 && diff < min_diff {
            min_diff = diff;
        }
    }
    if min_diff == i64::MAX { 1 } else { min_diff }
}

fn synthesize_coverage_from_candles(candles: &[Candle]) -> Coverage {
    let mut cov = Coverage::new();
    if candles.is_empty() {
        return cov;
    }
    let threshold = candle_split_threshold_ms(effective_interval_ms(candles));
    let mut run_start = candles[0].open_time.timestamp_millis();
    let mut run_end = candles[0].close_time.timestamp_millis();
    for c in &candles[1..] {
        let next_open = c.open_time.timestamp_millis();
        if next_open - run_end > threshold {
            cov.add(run_start, run_end);
            run_start = next_open;
        }
        run_end = c.close_time.timestamp_millis();
    }
    cov.add(run_start, run_end);
    cov
}

/// Same scan as `synthesize_coverage_from_candles` but operating on the
/// packed on-disk representation — avoids deserializing millions of 1m
/// records just to read their timestamps.
fn synthesize_coverage_from_1m_raw(data: &[u8], count: usize) -> Coverage {
    let mut cov = Coverage::new();
    if count == 0 {
        return cov;
    }
    // 1m candles: each record is RECORD_SIZE (64) bytes; the first 8 are
    // open_time_ms, next 8 are close_time_ms (CandleRecord layout).
    let read_open = |i: usize| -> i64 {
        let off = i * RECORD_SIZE;
        i64::from_le_bytes(data[off..off + 8].try_into().unwrap())
    };
    let read_close = |i: usize| -> i64 {
        let off = i * RECORD_SIZE;
        i64::from_le_bytes(data[off + 8..off + 16].try_into().unwrap())
    };

    // Use the *minimum* consecutive open-time difference. Picking just
    // the first pair would break if those two records bracket a real
    // hole — the inferred interval would be inflated and the split
    // threshold would mask interior gaps. See
    // `effective_interval_ms` for the same rationale on deserialized
    // candles.
    let interval_ms = if count >= 2 {
        let mut min_diff = i64::MAX;
        for i in 0..count - 1 {
            let diff = read_open(i + 1) - read_open(i);
            if diff > 0 && diff < min_diff {
                min_diff = diff;
            }
        }
        if min_diff == i64::MAX { 1 } else { min_diff }
    } else {
        (read_close(0) - read_open(0)).max(1)
    };
    let threshold = candle_split_threshold_ms(interval_ms);

    let mut run_start = read_open(0);
    let mut run_end = read_close(0);
    for i in 1..count {
        let next_open = read_open(i);
        if next_open - run_end > threshold {
            cov.add(run_start, run_end);
            run_start = next_open;
        }
        run_end = read_close(i);
    }
    cov.add(run_start, run_end);
    cov
}

/// Funding ticks nominally land every 8 hours. A gap exceeding 9 hours
/// is treated as a coverage split; normal ticks are coalesced.
const FUNDING_SPLIT_THRESHOLD_MS: i64 = 9 * 3600 * 1000;

fn synthesize_coverage_from_funding(rates: &[FundingRate]) -> Coverage {
    let mut cov = Coverage::new();
    if rates.is_empty() {
        return cov;
    }
    let mut run_start = rates[0].timestamp.timestamp_millis();
    let mut last_ts = run_start;
    for r in &rates[1..] {
        let ts = r.timestamp.timestamp_millis();
        if ts - last_ts > FUNDING_SPLIT_THRESHOLD_MS {
            // Half-open end = last_ts + 1 includes the final tick of
            // the run instead of leaving it as an off-by-one boundary.
            cov.add(run_start, last_ts + 1);
            run_start = ts;
        }
        last_ts = ts;
    }
    cov.add(run_start, last_ts + 1);
    cov
}

/// Read a 1m .bin file into a `ValidatedCache`. Returns `None` if the file
/// is missing, too short, has the wrong magic, or fails CRC validation.
///
/// Reuses the buffer returned by `fs::read` rather than copying the records
/// region into a fresh `Vec`.
fn load_1m_raw(path: &std::path::Path) -> Option<ValidatedCache> {
    let mut raw = match fs::read(path) {
        Ok(b) => b,
        _ => return None,
    };

    // Validate header + CRC while holding an immutable borrow of `raw`.
    let count = {
        let content = validate_and_strip_footer(&raw, path)?;
        let magic = u32::from_le_bytes(content[0..4].try_into().unwrap());
        if magic != FILE_MAGIC {
            eprintln!(
                "WARNING: {} has wrong magic — discarding.",
                path.display()
            );
            let _ = fs::remove_file(path);
            return None;
        }
        let file_count = u64::from_le_bytes(content[4..12].try_into().unwrap()) as usize;
        let file_data = &content[HEADER_SIZE..];
        let available = file_data.len() / RECORD_SIZE;
        if available < file_count {
            eprintln!(
                "WARNING: {} truncated — header expects {file_count} records, payload holds {available}. Discarding.",
                path.display()
            );
            let _ = fs::remove_file(path);
            return None;
        }
        file_count
    };

    // Drop the header prefix so `data[0..RECORD_SIZE]` is the first record —
    // matches the layout that `binary_search_records` and
    // `load_range_from_file` expect. The payload may include trailing bytes
    // beyond `count * RECORD_SIZE` if the file is "over-sized", but
    // `binary_search_records` respects `count` and ignores any trailing bytes.
    raw.truncate(HEADER_SIZE + count * RECORD_SIZE);
    raw.drain(..HEADER_SIZE);

    Some(ValidatedCache {
        data: Arc::new(raw),
        count,
    })
}

/// Binary search over on-disk CandleRecords.
///
/// If `by_close_time` is true, finds the first record where close_time_ms >= target_ms.
/// If false, finds the first record where open_time_ms >= target_ms.
/// Records are sorted by open_time (and thus close_time) on disk.
fn binary_search_records(data: &[u8], count: usize, target_ms: i64, by_close_time: bool) -> usize {
    debug_assert!(
        count
            .checked_mul(RECORD_SIZE)
            .map(|n| n <= data.len())
            .unwrap_or(false),
        "binary_search_records: count={count} * RECORD_SIZE exceeds data.len()={}",
        data.len()
    );
    let mut lo = 0usize;
    let mut hi = count;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let offset = mid * RECORD_SIZE;
        // Read just the timestamp field we need (first 8 bytes = open_time, next 8 = close_time)
        let ts_offset = if by_close_time { offset + 8 } else { offset };
        let ts = i64::from_le_bytes(data[ts_offset..ts_offset + 8].try_into().unwrap());
        if ts < target_ms {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn candle(start: DateTime<Utc>, interval: Duration, price: f64) -> Candle {
        Candle {
            open_time: start,
            close_time: start + interval,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price + 0.5,
            volume: price * 10.0,
            taker_buy_volume: price * 5.0,
        }
    }

    #[test]
    fn get_range_ref_matches_owned_get_range() {
        let symbol = "UT_CANDLE_RANGE_REF_1H";
        let interval = Duration::hours(1);
        let candles = vec![
            candle(dt(2024, 1, 1, 0, 0), interval, 100.0),
            candle(dt(2024, 1, 1, 1, 0), interval, 101.0),
            candle(dt(2024, 1, 1, 2, 0), interval, 102.0),
        ];

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);

        let owned = store.get_range(symbol, "1h", dt(2024, 1, 1, 0, 30), dt(2024, 1, 1, 3, 0));
        let borrowed =
            store.get_range_ref(symbol, "1h", dt(2024, 1, 1, 0, 30), dt(2024, 1, 1, 3, 0));

        assert_eq!(
            serde_json::to_value(&owned).unwrap(),
            serde_json::to_value(&borrowed).unwrap()
        );
    }

    #[test]
    fn get_all_ref_returns_full_cached_slice() {
        let symbol = "UT_CANDLE_ALL_REF_4H";
        let interval = Duration::hours(4);
        let candles = vec![
            candle(dt(2024, 1, 1, 0, 0), interval, 100.0),
            candle(dt(2024, 1, 1, 4, 0), interval, 101.0),
        ];

        let mut store = CandleStore::new();
        store.put(symbol, "4h", &candles);

        let all = store.get_all_ref(symbol, "4h");
        assert_eq!(
            serde_json::to_value(all).unwrap(),
            serde_json::to_value(&candles).unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "get_all_ref not supported for 1m intervals")]
    fn get_all_ref_panics_for_1m() {
        let mut store = CandleStore::new();
        let _ = store.get_all_ref("UT_CANDLE_ALL_REF_1M", "1m");
    }

    /// Pin the CRC polynomial. "123456789" → 0xCBF43926 is the canonical
    /// IEEE 802.3 / ITU-T V.42 check value; a future dep swap that silently
    /// changes the polynomial or XOR masks would flip this.
    #[test]
    fn crc32_known_answer() {
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
        assert_eq!(crc32(b""), 0x0000_0000);
        assert_eq!(crc32(&[0u8; 32]), 0x190A_55AD);
    }

    /// I/O-backed regression: put() writes a file with a trailer CRC that
    /// must match crc32() over the content slice. Catches any disagreement
    /// between the write path and the read path in one shot.
    #[test]
    fn crc32_trailer_roundtrip_on_disk() {
        let symbol = "UT_CANDLE_CRC_ROUNDTRIP_1H";
        let interval = Duration::hours(1);
        let candles: Vec<Candle> = (0..64)
            .map(|i| {
                candle(
                    dt(2024, 1, 1, 0, 0) + interval * i,
                    interval,
                    100.0 + i as f64,
                )
            })
            .collect();

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);

        let path = store.file_path(symbol, "1h");
        let bytes = fs::read(&path).expect("file exists after put");
        assert!(bytes.len() > HEADER_SIZE + FOOTER_SIZE);

        let content = &bytes[..bytes.len() - FOOTER_SIZE];
        let stored =
            u32::from_le_bytes(bytes[bytes.len() - FOOTER_SIZE..].try_into().unwrap());
        assert_eq!(
            crc32(content),
            stored,
            "trailer CRC must match crc32(content)"
        );

        // Read path must accept the file we just wrote. A *fresh* store is
        // required — the one that did put() has the candles in self.mem and
        // would never touch disk. A new CandleStore starts with an empty
        // cache, so get_range_ref hits load_from_file → validate_and_strip_footer
        // → crc32(content), which is what we actually want to exercise.
        let mut fresh = CandleStore::new();
        let reloaded = fresh.get_range_ref(
            symbol,
            "1h",
            candles[0].open_time,
            candles.last().unwrap().close_time + interval,
        );
        assert_eq!(reloaded.len(), candles.len());
    }

    /// A .bin whose header count exceeds what the payload can actually hold
    /// is unsafe to trust partially — returning the fitting prefix would let
    /// a future coverage layer record the missing suffix as "covered" and
    /// suppress re-fetch. Loader must discard the file wholesale.
    #[test]
    fn candle_load_rejects_truncated_file() {
        let symbol = "UT_CANDLE_TRUNCATED_1H";
        let interval = Duration::hours(1);
        let candles: Vec<Candle> = (0..10)
            .map(|i| candle(dt(2024, 1, 1, 0, 0) + interval * i, interval, 100.0 + i as f64))
            .collect();

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);
        let path = store.file_path(symbol, "1h");
        assert!(path.exists(), "precondition: put() must write the file");

        // Forge a .bin that claims 100 records but only has the 10 we wrote.
        // Keep the CRC valid so the failure classifies as truncation, not
        // CRC corruption.
        let mut buf = Vec::new();
        buf.extend_from_slice(&FILE_MAGIC.to_le_bytes());
        buf.extend_from_slice(&100u64.to_le_bytes());
        buf.resize(HEADER_SIZE + candles.len() * RECORD_SIZE, 0);
        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        fs::write(&path, &buf).expect("write forged file");

        // Fresh store so the stale in-memory cache from put() isn't consulted.
        let mut fresh = CandleStore::new();
        let got = fresh.get_range(
            symbol,
            "1h",
            candles[0].open_time,
            candles.last().unwrap().close_time + interval,
        );
        assert!(
            got.is_empty(),
            "truncated file must be treated as empty, got {} candles",
            got.len()
        );
        assert!(
            !path.exists(),
            "truncated file must be removed so next fetch repopulates it"
        );
    }

    /// Funding file whose header claims records but whose payload is empty —
    /// must be discarded for the same reason as the candle case.
    #[test]
    fn funding_load_rejects_truncated_file() {
        let symbol = "UT_FUNDING_TRUNCATED";
        let path = {
            let store = CandleStore::new();
            store.funding_path(symbol)
        };

        // Header claims 5 records, payload is zero records. CRC remains valid
        // so the loader must detect the truncation itself.
        let mut buf = Vec::new();
        buf.extend_from_slice(&FUND_MAGIC.to_le_bytes());
        buf.extend_from_slice(&5u64.to_le_bytes());
        // No payload.
        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        fs::write(&path, &buf).expect("write forged funding file");

        let mut store = CandleStore::new();
        let got = store.get_funding_range(
            symbol,
            dt(2024, 1, 1, 0, 0),
            dt(2024, 12, 31, 0, 0),
        );
        assert!(
            got.is_empty(),
            "truncated funding file must be treated as empty, got {} rates",
            got.len()
        );
        assert!(
            !path.exists(),
            "truncated funding file must be removed so next fetch repopulates it"
        );
    }

    // -----------------------------------------------------------------
    // Phase 2: coverage integration
    // -----------------------------------------------------------------

    fn funding_rate(ts: DateTime<Utc>, rate: f64) -> FundingRate {
        FundingRate {
            timestamp: ts,
            funding_rate: rate,
            mark_price: None,
        }
    }

    /// Record coverage in one store, drop it, open a fresh store, and
    /// verify the coverage comes back intact from disk.
    #[test]
    fn record_candle_coverage_persists_across_store_instances() {
        let symbol = "UT_COV_PERSIST_1H";
        let interval = Duration::hours(1);
        let candles = vec![
            candle(dt(2024, 1, 1, 0, 0), interval, 100.0),
            candle(dt(2024, 1, 1, 1, 0), interval, 101.0),
        ];

        {
            let mut store = CandleStore::new();
            store.put(symbol, "1h", &candles);
            store.record_candle_coverage(symbol, "1h", dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 2, 0));
        }

        let mut fresh = CandleStore::new();
        let gaps = fresh.candle_coverage_gaps(symbol, "1h", dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 2, 0));
        assert!(
            gaps.is_empty(),
            "persisted coverage must report zero gaps; got {gaps:?}"
        );
        // A slice outside the recorded window is still uncovered.
        let gaps = fresh.candle_coverage_gaps(symbol, "1h", dt(2024, 1, 1, 2, 0), dt(2024, 1, 1, 3, 0));
        assert_eq!(gaps.len(), 1);

        // Cleanup so the test directory stays tidy.
        fresh.invalidate_candle(symbol, "1h");
    }

    /// The exact bug Phase 2 is designed to catch: non-contiguous backfill
    /// where edges match the request but a middle region is missing. Edge-
    /// only gap detection would report no gaps here; coverage must report
    /// the middle one.
    #[test]
    fn candle_coverage_gaps_detects_non_contiguous_backfill() {
        let symbol = "UT_COV_NONCONTIG_1H";
        let interval = Duration::hours(1);
        // Two disjoint runs of candles: [Jan 1 00:00 – Jan 1 03:00) and
        // [Jan 2 00:00 – Jan 2 03:00). Put them in, then record coverage
        // matching what each run actually probed — not the edge-only
        // union.
        let run_a: Vec<Candle> = (0..3)
            .map(|i| candle(dt(2024, 1, 1, i as u32, 0), interval, 100.0 + i as f64))
            .collect();
        let run_b: Vec<Candle> = (0..3)
            .map(|i| candle(dt(2024, 1, 2, i as u32, 0), interval, 200.0 + i as f64))
            .collect();

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &run_a);
        store.put(symbol, "1h", &run_b);
        store.record_candle_coverage(symbol, "1h", dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 3, 0));
        store.record_candle_coverage(symbol, "1h", dt(2024, 1, 2, 0, 0), dt(2024, 1, 2, 3, 0));

        // Request spans both runs plus the Jan 1 03:00 – Jan 2 00:00 middle.
        let gaps = store.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 2, 3, 0),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 3, 0), dt(2024, 1, 2, 0, 0))],
            "must surface the interior gap that edge-only detection misses"
        );

        store.invalidate_candle(symbol, "1h");
    }

    /// Corrupting the `.bin` after persisting coverage must invalidate
    /// both files — otherwise the sidecar would claim coverage for data
    /// that no longer exists.
    #[test]
    fn corrupting_bin_invalidates_cov_sidecar() {
        let symbol = "UT_COV_BIN_CORRUPT_1H";
        let interval = Duration::hours(1);
        let candles: Vec<Candle> = (0..5)
            .map(|i| candle(dt(2024, 1, 1, i as u32, 0), interval, 100.0 + i as f64))
            .collect();

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);
        store.record_candle_coverage(symbol, "1h", dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 5, 0));
        drop(store);

        // Both files should exist at this point.
        let probe = CandleStore::new();
        let bin_path = probe.file_path(symbol, "1h");
        let cov_path = probe.coverage_path_candle(symbol, "1h");
        drop(probe);
        assert!(bin_path.exists(), "precondition: .bin must exist");
        assert!(cov_path.exists(), "precondition: .cov must exist");

        // Forge a truncated .bin that claims more records than it holds.
        let mut buf = Vec::new();
        buf.extend_from_slice(&FILE_MAGIC.to_le_bytes());
        buf.extend_from_slice(&100u64.to_le_bytes());
        buf.resize(HEADER_SIZE + 5 * RECORD_SIZE, 0);
        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        fs::write(&bin_path, &buf).expect("write forged bin");

        // Fresh store so in-memory caches don't short-circuit disk access.
        let mut fresh = CandleStore::new();
        let got = fresh.get_range(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 5, 0),
        );
        assert!(got.is_empty(), "corrupted .bin must read as empty");
        assert!(!bin_path.exists(), ".bin must be removed");
        assert!(
            !cov_path.exists(),
            ".cov must also be removed so coverage doesn't lie about missing data"
        );

        // After invalidation, coverage reports the full range as a gap.
        let gaps = fresh.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 5, 0),
        );
        assert_eq!(gaps.len(), 1, "full range must now be uncovered: {gaps:?}");
    }

    /// Auto-heal migration: when a pre-existing `.bin` has no matching
    /// `.cov`, synthesize coverage by scanning for internal gaps that
    /// exceed `interval_ms * 1.5`.
    #[test]
    fn synthesize_candle_coverage_splits_on_internal_gap() {
        let symbol = "UT_COV_AUTOHEAL_1H";
        let interval = Duration::hours(1);
        // Two contiguous runs with a 5-hour gap in between — well beyond
        // the 1.5-hour tolerance for 1h intervals.
        let mut candles = Vec::new();
        for i in 0..3 {
            candles.push(candle(dt(2024, 1, 1, i as u32, 0), interval, 100.0 + i as f64));
        }
        for i in 0..3 {
            candles.push(candle(dt(2024, 1, 1, 8 + i as u32, 0), interval, 200.0 + i as f64));
        }

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);
        // Explicitly remove the .cov so synthesize runs.
        let cov_path = store.coverage_path_candle(symbol, "1h");
        let _ = fs::remove_file(&cov_path);
        // Also clear in-memory coverage so the next query re-loads.
        store.candle_coverage.remove(&(symbol.to_string(), "1h".to_string()));

        // Request covering the full 11-hour span should report the middle
        // gap once synthesis splits the coverage.
        let gaps = store.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 11, 0),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 3, 0), dt(2024, 1, 1, 8, 0))],
            "auto-heal must split coverage at the 5-hour internal gap"
        );

        store.invalidate_candle(symbol, "1h");
    }

    /// Coverage without a backing `.bin` is a first-class state, not a
    /// stale artifact: it records a probe that returned no rows (empty
    /// trailing tail, pre-listing range, genuinely empty symbol). A
    /// fresh store must read the sidecar back and report zero gaps
    /// within the recorded range.
    #[test]
    fn empty_probe_coverage_persists_across_store_instances() {
        let symbol = "UT_COV_EMPTY_PROBE_1H";

        // First process: record an empty probe and drop the store.
        {
            let mut store = CandleStore::new();
            // Make sure nothing stale remains from a prior test run.
            store.invalidate_candle(symbol, "1h");
            store.record_candle_coverage(
                symbol,
                "1h",
                dt(2024, 1, 1, 0, 0),
                dt(2024, 1, 1, 6, 0),
            );
        }

        // Second process: the empty-probe coverage must be honored.
        let mut fresh = CandleStore::new();
        let gaps = fresh.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 6, 0),
        );
        assert!(
            gaps.is_empty(),
            "empty probe must survive restart; got {gaps:?}"
        );
        // But anything outside the probed range is still a gap.
        let gaps = fresh.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 10, 0),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 6, 0), dt(2024, 1, 1, 10, 0))]
        );

        fresh.invalidate_candle(symbol, "1h");
    }

    /// Funding parity for the empty-probe case — this is the original
    /// "sparse funding re-fetches every run" bug, now locked in across
    /// process restarts.
    #[test]
    fn empty_funding_probe_persists_across_store_instances() {
        let symbol = "UT_COV_EMPTY_FUND";

        {
            let mut store = CandleStore::new();
            store.invalidate_funding(symbol);
            store.record_funding_coverage(
                symbol,
                dt(2024, 1, 1, 0, 0),
                dt(2024, 1, 1, 12, 0),
            );
        }

        let mut fresh = CandleStore::new();
        let gaps =
            fresh.funding_coverage_gaps(symbol, dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 12, 0));
        assert!(gaps.is_empty(), "empty funding probe must survive restart");

        fresh.invalidate_funding(symbol);
    }

    /// Auto-heal interval inference must not be fooled by the first pair
    /// bracketing a real hole. With 1h candles at 00:00, 05:00, 06:00
    /// the naive "first two open times" heuristic infers 5h as the
    /// interval and merges the 4h hole into a single run. Using the
    /// minimum consecutive diff across all pairs (1h) correctly splits
    /// coverage at the 4h gap.
    #[test]
    fn synthesize_candle_coverage_uses_min_diff_not_first_pair() {
        let symbol = "UT_COV_MIN_DIFF_1H";
        let interval = Duration::hours(1);
        let candles = vec![
            candle(dt(2024, 1, 1, 0, 0), interval, 100.0),
            candle(dt(2024, 1, 1, 5, 0), interval, 101.0),
            candle(dt(2024, 1, 1, 6, 0), interval, 102.0),
        ];

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);
        // Force synthesis by clearing any cached or persisted .cov.
        let cov_path = store.coverage_path_candle(symbol, "1h");
        let _ = fs::remove_file(&cov_path);
        store.candle_coverage.remove(&(symbol.to_string(), "1h".to_string()));

        let gaps = store.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 7, 0),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 1, 0), dt(2024, 1, 1, 5, 0))],
            "min-diff inference must surface the 4h hole that first-pair inference would have merged"
        );

        store.invalidate_candle(symbol, "1h");
    }

    /// Funding parity: recording coverage for a probed-but-empty trailing
    /// range must suppress re-probe on the next query. This is the
    /// original "sparse funding re-fetches every run" fix.
    #[test]
    fn record_funding_coverage_suppresses_empty_trailing_refetch() {
        let symbol = "UT_COV_FUND_TAIL";

        let mut store = CandleStore::new();
        store.put_funding(
            symbol,
            &[funding_rate(dt(2024, 1, 1, 0, 0), 0.01)],
        );
        // Probe extends to Jan 1 12:00 but only found a tick at 00:00.
        // Record the full probed range anyway.
        store.record_funding_coverage(symbol, dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 12, 0));

        // Subsequent query for the same window must report no gaps even
        // though only one tick is actually stored.
        let gaps = store.funding_coverage_gaps(
            symbol,
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 12, 0),
        );
        assert!(gaps.is_empty(), "probed-but-empty tail must not re-probe: {gaps:?}");

        // Extending past the recorded range picks up the new tail only.
        let gaps = store.funding_coverage_gaps(
            symbol,
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 20, 0),
        );
        assert_eq!(gaps, vec![(dt(2024, 1, 1, 12, 0), dt(2024, 1, 1, 20, 0))]);

        store.invalidate_funding(symbol);
    }

    /// Funding migration: the `last_tick + 1ms` upper bound means a single
    /// stored tick covers exactly its instant — not zero-width.
    #[test]
    fn synthesize_funding_coverage_single_tick_is_not_empty() {
        let symbol = "UT_COV_FUND_SINGLE";

        let mut store = CandleStore::new();
        store.put_funding(
            symbol,
            &[funding_rate(dt(2024, 1, 1, 0, 0), 0.01)],
        );
        // Ensure synthesis runs (no .cov).
        let cov_path = store.coverage_path_funding(symbol);
        let _ = fs::remove_file(&cov_path);
        store.funding_coverage.remove(symbol);

        // The instant [0, 1ms) must be covered.
        let gaps = store.funding_coverage_gaps(
            symbol,
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 0, 0) + Duration::milliseconds(1),
        );
        assert!(
            gaps.is_empty(),
            "single-tick funding must cover its instant via +1ms upper bound: {gaps:?}"
        );

        store.invalidate_funding(symbol);
    }

    /// invalidate_candle drops everything: in-memory caches, .bin, .cov.
    #[test]
    fn invalidate_candle_is_thorough() {
        let symbol = "UT_COV_INVALIDATE_1H";
        let interval = Duration::hours(1);
        let candles = vec![candle(dt(2024, 1, 1, 0, 0), interval, 100.0)];

        let mut store = CandleStore::new();
        store.put(symbol, "1h", &candles);
        store.record_candle_coverage(symbol, "1h", dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 1, 0));

        let bin_path = store.file_path(symbol, "1h");
        let cov_path = store.coverage_path_candle(symbol, "1h");
        let key = (symbol.to_string(), "1h".to_string());
        assert!(bin_path.exists());
        assert!(cov_path.exists());
        assert!(store.mem.contains_key(&key));
        assert!(store.candle_coverage.contains_key(&key));

        store.invalidate_candle(symbol, "1h");

        assert!(!bin_path.exists(), ".bin removed");
        assert!(!cov_path.exists(), ".cov removed");
        assert!(!store.mem.contains_key(&key), "mem entry cleared");
        assert!(!store.candle_coverage.contains_key(&key), "coverage entry cleared");
    }

    // -----------------------------------------------------------------
    // Corruption-vs-coverage ordering: coverage lookups must validate
    // the `.bin` before trusting the `.cov`. Otherwise a corrupt `.bin`
    // plus a still-valid `.cov` would report "no gaps" and suppress the
    // refetch that corruption requires.
    // -----------------------------------------------------------------

    /// A forged truncated `.bin` paired with a valid `.cov` must produce
    /// "full range is a gap" on the first `candle_coverage_gaps` call,
    /// not "no gaps". Exercises the load-before-trust ordering in
    /// `load_or_synthesize_candle_coverage`.
    #[test]
    fn candle_coverage_gaps_invalidates_stale_cov_over_corrupt_bin() {
        let symbol = "UT_COV_CORRUPT_ORDERING_1H";
        let interval = Duration::hours(1);
        let candles: Vec<Candle> = (0..5)
            .map(|i| candle(dt(2024, 1, 1, i as u32, 0), interval, 100.0 + i as f64))
            .collect();

        // Populate both .bin and .cov cleanly, then drop the store so
        // the in-memory caches don't short-circuit disk access.
        {
            let mut store = CandleStore::new();
            store.put(symbol, "1h", &candles);
            store.record_candle_coverage(
                symbol,
                "1h",
                dt(2024, 1, 1, 0, 0),
                dt(2024, 1, 1, 5, 0),
            );
        }

        // Forge a truncated .bin (valid CRC so the failure classifies
        // as truncation, not CRC corruption). Leave the .cov intact.
        let probe = CandleStore::new();
        let bin_path = probe.file_path(symbol, "1h");
        let cov_path = probe.coverage_path_candle(symbol, "1h");
        drop(probe);
        let mut buf = Vec::new();
        buf.extend_from_slice(&FILE_MAGIC.to_le_bytes());
        buf.extend_from_slice(&100u64.to_le_bytes());
        buf.resize(HEADER_SIZE + 5 * RECORD_SIZE, 0);
        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        fs::write(&bin_path, &buf).expect("write forged bin");
        assert!(cov_path.exists(), "precondition: .cov is still present");

        // Fresh store — no warm caches. First coverage-gaps call must
        // validate, invalidate, and surface the full range as a gap.
        let mut fresh = CandleStore::new();
        let gaps = fresh.candle_coverage_gaps(
            symbol,
            "1h",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 5, 0),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 5, 0))],
            "corrupt .bin must invalidate .cov BEFORE coverage is consulted"
        );
        assert!(!bin_path.exists(), "corrupt .bin must be removed");
        assert!(!cov_path.exists(), "stale .cov must be removed");
    }

    /// Funding parity for the corruption-ordering invariant.
    #[test]
    fn funding_coverage_gaps_invalidates_stale_cov_over_corrupt_bin() {
        let symbol = "UT_COV_CORRUPT_ORDERING_FUND";

        {
            let mut store = CandleStore::new();
            store.put_funding(
                symbol,
                &[funding_rate(dt(2024, 1, 1, 0, 0), 0.01)],
            );
            store.record_funding_coverage(
                symbol,
                dt(2024, 1, 1, 0, 0),
                dt(2024, 1, 1, 12, 0),
            );
        }

        let probe = CandleStore::new();
        let bin_path = probe.funding_path(symbol);
        let cov_path = probe.coverage_path_funding(symbol);
        drop(probe);
        // Forge a funding .bin with header_count > payload records.
        let mut buf = Vec::new();
        buf.extend_from_slice(&FUND_MAGIC.to_le_bytes());
        buf.extend_from_slice(&50u64.to_le_bytes());
        buf.resize(HEADER_SIZE + 1 * FUND_RECORD_SIZE, 0);
        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        fs::write(&bin_path, &buf).expect("write forged funding bin");
        assert!(cov_path.exists(), "precondition: .cov is still present");

        let mut fresh = CandleStore::new();
        let gaps = fresh.funding_coverage_gaps(
            symbol,
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 12, 0),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 12, 0))],
            "corrupt funding .bin must invalidate .cov BEFORE coverage is consulted"
        );
        assert!(!bin_path.exists());
        assert!(!cov_path.exists());
    }

    /// `preload_1m` runs in worker threads that can't invalidate on
    /// their own. The post-parallel reconciliation step must clean up
    /// any symbol that failed to load — both the stale `.cov` and any
    /// cached coverage — so a later `candle_coverage_gaps` call doesn't
    /// trust stale metadata.
    #[test]
    fn preload_1m_invalidates_failed_symbols() {
        let symbol = "UT_COV_PRELOAD_CORRUPT_1M";

        // Build a valid .bin + .cov, then corrupt the .bin. Use 1m.
        let interval = Duration::minutes(1);
        let candles: Vec<Candle> = (0..5)
            .map(|i| candle(dt(2024, 1, 1, 0, i as u32), interval, 100.0 + i as f64))
            .collect();
        {
            let mut store = CandleStore::new();
            store.put(symbol, "1m", &candles);
            store.record_candle_coverage(
                symbol,
                "1m",
                dt(2024, 1, 1, 0, 0),
                dt(2024, 1, 1, 0, 5),
            );
        }

        let probe = CandleStore::new();
        let bin_path = probe.file_path(symbol, "1m");
        let cov_path = probe.coverage_path_candle(symbol, "1m");
        drop(probe);

        // Forge a truncated 1m .bin (magic + header + too-few records + valid CRC).
        let mut buf = Vec::new();
        buf.extend_from_slice(&FILE_MAGIC.to_le_bytes());
        buf.extend_from_slice(&1000u64.to_le_bytes());
        buf.resize(HEADER_SIZE + 5 * RECORD_SIZE, 0);
        let checksum = crc32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        fs::write(&bin_path, &buf).expect("write forged 1m bin");
        assert!(cov_path.exists(), "precondition: .cov is still present");

        let mut fresh = CandleStore::new();
        fresh.preload_1m(&[symbol.to_string()]);

        assert!(
            !bin_path.exists(),
            "preload must delete corrupt .bin via load_1m_raw"
        );
        assert!(
            !cov_path.exists(),
            "preload must reconcile: failed load → invalidate .cov"
        );

        // A follow-up coverage query must report the full range as a gap.
        let gaps = fresh.candle_coverage_gaps(
            symbol,
            "1m",
            dt(2024, 1, 1, 0, 0),
            dt(2024, 1, 1, 0, 5),
        );
        assert_eq!(
            gaps,
            vec![(dt(2024, 1, 1, 0, 0), dt(2024, 1, 1, 0, 5))],
            "coverage must be empty after preload-triggered invalidation"
        );
    }
}
