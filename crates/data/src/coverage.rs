//! Coverage: explicit tracking of probed time intervals per data file.
//!
//! A [`Coverage`] is a sorted, non-overlapping, non-touching set of
//! half-open `[start_ms, end_ms)` millisecond intervals. It records the
//! ranges that have definitively been asked of the upstream API, as
//! opposed to the ranges the stored data happens to span.
//!
//! The distinction matters for two bugs the edge-only gap detector in
//! `ensure_*` has today:
//!
//! 1. Non-contiguous backfill: runs covering `[A, B]` and `[C, D]` with
//!    `B < C` produce stored data whose edges match later queries, but
//!    the `[B, C]` interior is never detected or re-fetched.
//! 2. Trailing empty fetches: funding's sparse 8-hour cadence means a
//!    fetch for `[last_tick, now)` often returns no rows, so
//!    `last_close`/`last_ts` never advances and the next run re-probes
//!    the identical empty range forever.
//!
//! Recording the *probed* range (not the data range) collapses both
//! failure modes into the same correct behavior: coverage marks what was
//! asked; `gaps()` returns only the uncovered part of a request.
//!
//! ## Invariants (enforced by `add` and `from_bytes`)
//!
//! - Intervals are half-open: `[start, end)`.
//! - Every interval has `start < end`. Zero-width adds are no-ops.
//! - Intervals are stored sorted by `start` ascending.
//! - No two intervals overlap or touch; `add((A, B))` followed by
//!   `add((B, C))` coalesces into a single `(A, C)`.
//!
//! ## On-disk format
//!
//! ```text
//! offset  size  field
//! ------  ----  -----
//!   0     4     magic = 0x5256_4F43 ("COVR" little-endian)
//!   4     4     version = 1
//!   8     8     count (u64, number of intervals)
//!  16     N*16  records: (i64 start_ms, i64 end_ms) in little-endian
//!  end    4     CRC32 (ISO 3309) of every byte preceding it
//! ```
//!
//! `from_bytes` rejects any file that fails magic, version, length-vs-
//! count, CRC, or interval-invariant checks. Matches the existing
//! `.bin` fail-closed pattern.

use std::fs;
use std::io::Write;
use std::path::Path;

const COV_MAGIC: u32 = 0x5256_4F43; // "COVR" little-endian
const COV_VERSION: u32 = 1;
const COV_HEADER_SIZE: usize = 4 + 4 + 8; // magic + version + count
const COV_RECORD_SIZE: usize = 16; // i64 start + i64 end
const COV_FOOTER_SIZE: usize = 4; // CRC32

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    intervals: Vec<(i64, i64)>,
}

impl Coverage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `[start_ms, end_ms)` as probed.
    ///
    /// Coalesces with any adjacent or overlapping existing intervals.
    /// `start_ms >= end_ms` is a no-op (zero-width ranges carry no
    /// information — use `add(T, T+1)` to record the single instant `T`).
    pub fn add(&mut self, start_ms: i64, end_ms: i64) {
        if start_ms >= end_ms {
            return;
        }

        // First index whose end_i is >= start_ms — the start of the
        // overlap/touch window. (Intervals with end_i < start_ms sit
        // strictly to the left of the new range.)
        let lo = self
            .intervals
            .partition_point(|&(_, b)| b < start_ms);
        // First index whose start_i > end_ms — one past the end of the
        // window. (Intervals with start_i <= end_ms either overlap or
        // touch the new range.)
        let hi_exclusive = self
            .intervals
            .partition_point(|&(a, _)| a <= end_ms);

        if lo == hi_exclusive {
            // No overlap or touch — insert at the sorted position.
            self.intervals.insert(lo, (start_ms, end_ms));
            return;
        }

        // Coalesce `intervals[lo..hi_exclusive]` with the new range.
        let merged_start = self.intervals[lo].0.min(start_ms);
        let merged_end = self.intervals[hi_exclusive - 1].1.max(end_ms);
        self.intervals
            .splice(lo..hi_exclusive, std::iter::once((merged_start, merged_end)));
    }

    /// Compute the sub-ranges of `[start_ms, end_ms)` not covered.
    ///
    /// Returns a vector of `(gap_start, gap_end)` half-open pairs, each
    /// with `gap_start < gap_end`. An empty return value means the
    /// request is fully covered. `start_ms >= end_ms` returns empty.
    pub fn gaps(&self, start_ms: i64, end_ms: i64) -> Vec<(i64, i64)> {
        if start_ms >= end_ms {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut cursor = start_ms;
        for &(a, b) in &self.intervals {
            if b <= cursor {
                continue;
            }
            if a >= end_ms {
                break;
            }
            if a > cursor {
                out.push((cursor, a));
            }
            cursor = cursor.max(b);
            if cursor >= end_ms {
                break;
            }
        }
        if cursor < end_ms {
            out.push((cursor, end_ms));
        }
        out
    }

    /// True if every millisecond in `[start_ms, end_ms)` is covered.
    /// Vacuously true for zero-width ranges (`start_ms >= end_ms`).
    pub fn contains(&self, start_ms: i64, end_ms: i64) -> bool {
        if start_ms >= end_ms {
            return true;
        }
        // Intervals are non-touching and non-overlapping, so if `[start,
        // end)` is covered it must lie inside a single interval. Locate
        // the last interval whose start <= start_ms and check its end.
        let idx = self.intervals.partition_point(|&(a, _)| a <= start_ms);
        if idx == 0 {
            return false;
        }
        let (_, b) = self.intervals[idx - 1];
        b >= end_ms
    }

    pub fn intervals(&self) -> &[(i64, i64)] {
        &self.intervals
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    /// Serialize to the binary format described at module level.
    pub fn to_bytes(&self) -> Vec<u8> {
        let count = self.intervals.len();
        let mut buf = Vec::with_capacity(
            COV_HEADER_SIZE + count * COV_RECORD_SIZE + COV_FOOTER_SIZE,
        );
        buf.extend_from_slice(&COV_MAGIC.to_le_bytes());
        buf.extend_from_slice(&COV_VERSION.to_le_bytes());
        buf.extend_from_slice(&(count as u64).to_le_bytes());
        for &(s, e) in &self.intervals {
            buf.extend_from_slice(&s.to_le_bytes());
            buf.extend_from_slice(&e.to_le_bytes());
        }
        let checksum = crc32fast::hash(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf
    }

    /// Parse and validate the binary format. Returns `None` on any
    /// structural failure: too short, bad magic, unsupported version,
    /// length-vs-count mismatch, CRC mismatch, or broken interval
    /// invariants (unsorted, overlapping, touching, or empty range).
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < COV_HEADER_SIZE + COV_FOOTER_SIZE {
            return None;
        }
        let (content, footer) = bytes.split_at(bytes.len() - COV_FOOTER_SIZE);
        let stored_crc = u32::from_le_bytes(footer.try_into().ok()?);
        if crc32fast::hash(content) != stored_crc {
            return None;
        }

        let magic = u32::from_le_bytes(content[0..4].try_into().ok()?);
        if magic != COV_MAGIC {
            return None;
        }
        let version = u32::from_le_bytes(content[4..8].try_into().ok()?);
        if version != COV_VERSION {
            return None;
        }
        let count_u64 = u64::from_le_bytes(content[8..16].try_into().ok()?);
        let records_bytes = &content[COV_HEADER_SIZE..];
        // Bound the payload size in u64 first so a forged huge count can't
        // overflow `count * COV_RECORD_SIZE` or smuggle past the length check
        // via a 32-bit usize truncation. After this equality holds, count is
        // bounded by `records_bytes.len() / COV_RECORD_SIZE`, which fits in
        // usize by construction.
        let expected_payload = count_u64.checked_mul(COV_RECORD_SIZE as u64)?;
        if records_bytes.len() as u64 != expected_payload {
            // Exact match required — refuse truncated, over-sized, and
            // overflow-forged payloads alike.
            return None;
        }
        let count = count_u64 as usize;

        let mut intervals = Vec::with_capacity(count);
        for i in 0..count {
            let off = i * COV_RECORD_SIZE;
            let s = i64::from_le_bytes(records_bytes[off..off + 8].try_into().ok()?);
            let e = i64::from_le_bytes(records_bytes[off + 8..off + 16].try_into().ok()?);
            if s >= e {
                return None;
            }
            if let Some(&(_, prev_end)) = intervals.last() {
                let _prev_end: i64 = prev_end;
                if _prev_end >= s {
                    // Unsorted, overlapping, or touching — all invariant
                    // violations.
                    return None;
                }
            }
            intervals.push((s, e));
        }

        Some(Self { intervals })
    }

    /// Convenience: load from a file path. Returns `None` if the file is
    /// missing, unreadable, or fails `from_bytes` validation. Does not
    /// delete corrupted files — the caller decides.
    pub fn load_from_file(path: &Path) -> Option<Self> {
        let bytes = fs::read(path).ok()?;
        Self::from_bytes(&bytes)
    }

    /// Atomically write to disk via tmp + rename. Caller is responsible
    /// for picking the path.
    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        let tmp = path.with_extension("tmp");
        let bytes = self.to_bytes();
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, path).or_else(|e| {
            let _ = fs::remove_file(&tmp);
            Err(e)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // add: coalescing + insertion
    // -----------------------------------------------------------------

    #[test]
    fn add_into_empty_inserts_single_interval() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        assert_eq!(cov.intervals(), &[(10, 20)]);
    }

    #[test]
    fn add_zero_width_is_noop() {
        let mut cov = Coverage::new();
        cov.add(10, 10);
        cov.add(20, 5);
        assert!(cov.intervals().is_empty());
    }

    #[test]
    fn add_single_instant_via_plus_one_covers_it() {
        let mut cov = Coverage::new();
        cov.add(100, 101); // the instant at t=100
        assert!(cov.contains(100, 101));
        // the next instant is not covered
        assert!(!cov.contains(101, 102));
        // gaps at [100, 101) is empty
        assert!(cov.gaps(100, 101).is_empty());
    }

    #[test]
    fn add_disjoint_intervals_sorted() {
        let mut cov = Coverage::new();
        cov.add(30, 40);
        cov.add(10, 20);
        cov.add(50, 60);
        assert_eq!(cov.intervals(), &[(10, 20), (30, 40), (50, 60)]);
    }

    #[test]
    fn add_touching_right_coalesces() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        cov.add(20, 30);
        assert_eq!(cov.intervals(), &[(10, 30)]);
    }

    #[test]
    fn add_touching_left_coalesces() {
        let mut cov = Coverage::new();
        cov.add(20, 30);
        cov.add(10, 20);
        assert_eq!(cov.intervals(), &[(10, 30)]);
    }

    #[test]
    fn add_overlapping_extends_end() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        cov.add(15, 25);
        assert_eq!(cov.intervals(), &[(10, 25)]);
    }

    #[test]
    fn add_overlapping_extends_start() {
        let mut cov = Coverage::new();
        cov.add(15, 25);
        cov.add(10, 20);
        assert_eq!(cov.intervals(), &[(10, 25)]);
    }

    #[test]
    fn add_fully_contained_is_noop() {
        let mut cov = Coverage::new();
        cov.add(10, 30);
        cov.add(15, 25);
        assert_eq!(cov.intervals(), &[(10, 30)]);
    }

    #[test]
    fn add_superset_replaces() {
        let mut cov = Coverage::new();
        cov.add(15, 25);
        cov.add(10, 30);
        assert_eq!(cov.intervals(), &[(10, 30)]);
    }

    #[test]
    fn add_bridging_two_intervals_merges_all_three() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        cov.add(30, 40);
        cov.add(15, 35);
        assert_eq!(cov.intervals(), &[(10, 40)]);
    }

    #[test]
    fn add_bridging_by_touching_only_merges_all_three() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        cov.add(30, 40);
        cov.add(20, 30);
        assert_eq!(cov.intervals(), &[(10, 40)]);
    }

    #[test]
    fn add_spanning_many_coalesces() {
        let mut cov = Coverage::new();
        cov.add(10, 15);
        cov.add(20, 25);
        cov.add(30, 35);
        cov.add(40, 45);
        cov.add(5, 100);
        assert_eq!(cov.intervals(), &[(5, 100)]);
    }

    #[test]
    fn add_repeated_identical_intervals_stays_single() {
        let mut cov = Coverage::new();
        for _ in 0..10 {
            cov.add(10, 20);
        }
        assert_eq!(cov.intervals(), &[(10, 20)]);
    }

    #[test]
    fn add_reverse_order_sequence_yields_same_result_as_forward() {
        let mut fwd = Coverage::new();
        fwd.add(10, 20);
        fwd.add(30, 40);
        fwd.add(50, 60);

        let mut rev = Coverage::new();
        rev.add(50, 60);
        rev.add(30, 40);
        rev.add(10, 20);

        assert_eq!(fwd.intervals(), rev.intervals());
    }

    // -----------------------------------------------------------------
    // gaps
    // -----------------------------------------------------------------

    #[test]
    fn gaps_over_empty_coverage_returns_full_range() {
        let cov = Coverage::new();
        assert_eq!(cov.gaps(10, 20), vec![(10, 20)]);
    }

    #[test]
    fn gaps_fully_covered_returns_empty() {
        let mut cov = Coverage::new();
        cov.add(0, 100);
        assert!(cov.gaps(10, 50).is_empty());
    }

    #[test]
    fn gaps_with_single_interior_hole() {
        let mut cov = Coverage::new();
        cov.add(0, 50);
        cov.add(70, 100);
        assert_eq!(cov.gaps(0, 100), vec![(50, 70)]);
    }

    #[test]
    fn gaps_with_multiple_holes() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        cov.add(30, 40);
        cov.add(50, 60);
        assert_eq!(cov.gaps(0, 70), vec![(0, 10), (20, 30), (40, 50), (60, 70)]);
    }

    #[test]
    fn gaps_request_entirely_before_coverage() {
        let mut cov = Coverage::new();
        cov.add(100, 200);
        assert_eq!(cov.gaps(0, 50), vec![(0, 50)]);
    }

    #[test]
    fn gaps_request_entirely_after_coverage() {
        let mut cov = Coverage::new();
        cov.add(0, 100);
        assert_eq!(cov.gaps(200, 300), vec![(200, 300)]);
    }

    #[test]
    fn gaps_request_overlaps_partial_coverage_at_start() {
        let mut cov = Coverage::new();
        cov.add(20, 100);
        assert_eq!(cov.gaps(0, 50), vec![(0, 20)]);
    }

    #[test]
    fn gaps_request_overlaps_partial_coverage_at_end() {
        let mut cov = Coverage::new();
        cov.add(0, 50);
        assert_eq!(cov.gaps(20, 100), vec![(50, 100)]);
    }

    #[test]
    fn gaps_zero_width_request_returns_empty() {
        let mut cov = Coverage::new();
        cov.add(0, 100);
        assert!(cov.gaps(50, 50).is_empty());
        assert!(cov.gaps(200, 100).is_empty());
    }

    #[test]
    fn gaps_request_inside_single_interval_returns_empty() {
        let mut cov = Coverage::new();
        cov.add(0, 100);
        assert!(cov.gaps(30, 70).is_empty());
    }

    // -----------------------------------------------------------------
    // contains
    // -----------------------------------------------------------------

    #[test]
    fn contains_zero_width_is_vacuously_true() {
        let cov = Coverage::new();
        assert!(cov.contains(10, 10));
        assert!(cov.contains(20, 10));
    }

    #[test]
    fn contains_on_empty_coverage_is_false() {
        let cov = Coverage::new();
        assert!(!cov.contains(10, 20));
    }

    #[test]
    fn contains_exact_match_true() {
        let mut cov = Coverage::new();
        cov.add(10, 20);
        assert!(cov.contains(10, 20));
    }

    #[test]
    fn contains_strict_subrange_true() {
        let mut cov = Coverage::new();
        cov.add(0, 100);
        assert!(cov.contains(10, 20));
        assert!(cov.contains(0, 100));
        assert!(cov.contains(99, 100));
    }

    #[test]
    fn contains_across_gap_is_false() {
        let mut cov = Coverage::new();
        cov.add(0, 50);
        cov.add(60, 100);
        assert!(!cov.contains(40, 70));
    }

    #[test]
    fn contains_extends_past_end_is_false() {
        let mut cov = Coverage::new();
        cov.add(0, 50);
        assert!(!cov.contains(40, 60));
    }

    #[test]
    fn contains_starts_before_coverage_is_false() {
        let mut cov = Coverage::new();
        cov.add(10, 50);
        assert!(!cov.contains(0, 20));
    }

    // -----------------------------------------------------------------
    // round-trip serialization + validation
    // -----------------------------------------------------------------

    #[test]
    fn round_trip_empty() {
        let cov = Coverage::new();
        let bytes = cov.to_bytes();
        let parsed = Coverage::from_bytes(&bytes).expect("empty coverage must round-trip");
        assert_eq!(cov, parsed);
    }

    #[test]
    fn round_trip_multi_interval() {
        let mut cov = Coverage::new();
        cov.add(1_000, 2_000);
        cov.add(5_000, 6_000);
        cov.add(8_000, 9_000);
        let bytes = cov.to_bytes();
        let parsed = Coverage::from_bytes(&bytes).expect("must round-trip");
        assert_eq!(cov, parsed);
        assert_eq!(parsed.intervals(), &[(1_000, 2_000), (5_000, 6_000), (8_000, 9_000)]);
    }

    #[test]
    fn from_bytes_rejects_wrong_magic() {
        let mut cov = Coverage::new();
        cov.add(1, 2);
        let mut bytes = cov.to_bytes();
        bytes[0] ^= 0xFF; // corrupt magic
        // CRC will also be invalid but the point is from_bytes refuses
        // regardless.
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_crc_mismatch() {
        let mut cov = Coverage::new();
        cov.add(1, 2);
        let mut bytes = cov.to_bytes();
        // Corrupt a record byte, leave magic+version+count intact.
        let rec_off = COV_HEADER_SIZE;
        bytes[rec_off] ^= 0xFF;
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_unsupported_version() {
        let mut cov = Coverage::new();
        cov.add(1, 2);
        let mut bytes = cov.to_bytes();
        // Bump version past what we support.
        bytes[4..8].copy_from_slice(&999u32.to_le_bytes());
        // Recompute CRC so the rejection is specifically due to version.
        let content_len = bytes.len() - COV_FOOTER_SIZE;
        let crc = crc32fast::hash(&bytes[..content_len]);
        bytes[content_len..].copy_from_slice(&crc.to_le_bytes());
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_truncated_payload() {
        let mut cov = Coverage::new();
        cov.add(1, 2);
        cov.add(10, 20);
        let bytes = cov.to_bytes();
        // Drop the footer + last record entirely.
        let shortened = &bytes[..bytes.len() - COV_FOOTER_SIZE - COV_RECORD_SIZE];
        // Re-append a valid CRC over the shortened content so the rejection
        // is due to length, not CRC.
        let mut reforged = shortened.to_vec();
        let crc = crc32fast::hash(&reforged);
        reforged.extend_from_slice(&crc.to_le_bytes());
        // But the header still says count=2 while only 1 record is present.
        assert!(Coverage::from_bytes(&reforged).is_none());
    }

    #[test]
    fn from_bytes_rejects_oversized_payload() {
        let mut cov = Coverage::new();
        cov.add(1, 2);
        let bytes = cov.to_bytes();
        // Splice in an extra fake record between the last real one and
        // the CRC. This makes records_bytes.len() > count * RECORD_SIZE.
        let insert_at = bytes.len() - COV_FOOTER_SIZE;
        let mut inflated = bytes[..insert_at].to_vec();
        inflated.extend_from_slice(&[0u8; COV_RECORD_SIZE]);
        let crc = crc32fast::hash(&inflated);
        inflated.extend_from_slice(&crc.to_le_bytes());
        assert!(Coverage::from_bytes(&inflated).is_none());
    }

    /// A forged header claiming a count that would overflow
    /// `count * COV_RECORD_SIZE` must be rejected without panicking or
    /// allocating huge buffers. Guards Phase 2's contract: `.cov` parsing
    /// is fail-closed against arbitrary corrupt bytes.
    #[test]
    fn from_bytes_rejects_huge_count_overflow() {
        // Count so large that count * 16 overflows u64.
        let huge_count: u64 = (u64::MAX / (COV_RECORD_SIZE as u64)) + 1;

        let mut content = Vec::new();
        content.extend_from_slice(&COV_MAGIC.to_le_bytes());
        content.extend_from_slice(&COV_VERSION.to_le_bytes());
        content.extend_from_slice(&huge_count.to_le_bytes());
        // Plausible-looking payload so a naive parser that skipped the
        // overflow check would get past the magic/version check.
        content.extend_from_slice(&0i64.to_le_bytes());
        content.extend_from_slice(&100i64.to_le_bytes());

        let mut bytes = content.clone();
        let crc = crc32fast::hash(&content);
        bytes.extend_from_slice(&crc.to_le_bytes());

        // CRC is valid — the rejection must be specifically due to the
        // count overflow, not some upstream error.
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    /// Edge case: count exactly at the u64 overflow boundary. `u64::MAX
    /// / COV_RECORD_SIZE` does not overflow when multiplied by
    /// COV_RECORD_SIZE, but the payload can't actually hold that many
    /// records — so rejection must come from the length check.
    #[test]
    fn from_bytes_rejects_count_at_overflow_boundary() {
        let boundary_count: u64 = u64::MAX / (COV_RECORD_SIZE as u64);

        let mut content = Vec::new();
        content.extend_from_slice(&COV_MAGIC.to_le_bytes());
        content.extend_from_slice(&COV_VERSION.to_le_bytes());
        content.extend_from_slice(&boundary_count.to_le_bytes());
        content.extend_from_slice(&0i64.to_le_bytes());
        content.extend_from_slice(&100i64.to_le_bytes());

        let mut bytes = content.clone();
        let crc = crc32fast::hash(&content);
        bytes.extend_from_slice(&crc.to_le_bytes());

        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_unsorted_intervals() {
        // Forge raw bytes with intervals in the wrong order, valid CRC.
        let mut content = Vec::new();
        content.extend_from_slice(&COV_MAGIC.to_le_bytes());
        content.extend_from_slice(&COV_VERSION.to_le_bytes());
        content.extend_from_slice(&2u64.to_le_bytes());
        // Record 1: (100, 200)
        content.extend_from_slice(&100i64.to_le_bytes());
        content.extend_from_slice(&200i64.to_le_bytes());
        // Record 2: (0, 50) — out of order
        content.extend_from_slice(&0i64.to_le_bytes());
        content.extend_from_slice(&50i64.to_le_bytes());

        let mut bytes = content.clone();
        let crc = crc32fast::hash(&content);
        bytes.extend_from_slice(&crc.to_le_bytes());
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_overlapping_intervals() {
        let mut content = Vec::new();
        content.extend_from_slice(&COV_MAGIC.to_le_bytes());
        content.extend_from_slice(&COV_VERSION.to_le_bytes());
        content.extend_from_slice(&2u64.to_le_bytes());
        // (0, 100), (50, 150) — overlap
        content.extend_from_slice(&0i64.to_le_bytes());
        content.extend_from_slice(&100i64.to_le_bytes());
        content.extend_from_slice(&50i64.to_le_bytes());
        content.extend_from_slice(&150i64.to_le_bytes());

        let mut bytes = content.clone();
        let crc = crc32fast::hash(&content);
        bytes.extend_from_slice(&crc.to_le_bytes());
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_touching_intervals() {
        // Touching violates invariant (coalesce would have merged them).
        let mut content = Vec::new();
        content.extend_from_slice(&COV_MAGIC.to_le_bytes());
        content.extend_from_slice(&COV_VERSION.to_le_bytes());
        content.extend_from_slice(&2u64.to_le_bytes());
        content.extend_from_slice(&0i64.to_le_bytes());
        content.extend_from_slice(&100i64.to_le_bytes());
        content.extend_from_slice(&100i64.to_le_bytes());
        content.extend_from_slice(&200i64.to_le_bytes());

        let mut bytes = content.clone();
        let crc = crc32fast::hash(&content);
        bytes.extend_from_slice(&crc.to_le_bytes());
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_zero_width_interval() {
        let mut content = Vec::new();
        content.extend_from_slice(&COV_MAGIC.to_le_bytes());
        content.extend_from_slice(&COV_VERSION.to_le_bytes());
        content.extend_from_slice(&1u64.to_le_bytes());
        content.extend_from_slice(&50i64.to_le_bytes());
        content.extend_from_slice(&50i64.to_le_bytes());

        let mut bytes = content.clone();
        let crc = crc32fast::hash(&content);
        bytes.extend_from_slice(&crc.to_le_bytes());
        assert!(Coverage::from_bytes(&bytes).is_none());
    }

    #[test]
    fn from_bytes_rejects_too_short_input() {
        assert!(Coverage::from_bytes(&[]).is_none());
        assert!(Coverage::from_bytes(&[0u8; 8]).is_none());
    }

    #[test]
    fn load_and_write_roundtrip_via_filesystem() {
        let dir = std::env::temp_dir().join("coverage_rt_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("roundtrip.cov");

        let mut cov = Coverage::new();
        cov.add(100, 200);
        cov.add(300, 400);
        cov.write_to_file(&path).expect("write");

        let loaded = Coverage::load_from_file(&path).expect("load");
        assert_eq!(cov, loaded);

        // Write a truncated file and assert load_from_file returns None.
        let bytes = cov.to_bytes();
        let truncated = &bytes[..bytes.len() / 2];
        fs::write(&path, truncated).expect("write truncated");
        assert!(Coverage::load_from_file(&path).is_none());

        let _ = fs::remove_file(&path);
    }
}
