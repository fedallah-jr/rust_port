//! One-off candle continuity repair tool.
//!
//! Discovers the first exchange candle for each symbol, scans local candle
//! continuity from that point to a chosen end boundary, fetches missing
//! ranges, merges them into the flat candle store, and reports unresolved
//! gaps after a bounded number of repair passes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use claude_trader_data::{BinanceClient, CandleStore};
use claude_trader_models::{ms_to_dt, Candle, MarketType};

const DEFAULT_SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "XRPUSDT", "DOGEUSDT", "ADAUSDT", "AVAXUSDT",
    "LINKUSDT", "DOTUSDT", "LTCUSDT", "ARBUSDT", "OPUSDT", "SUIUSDT", "PEPEUSDT",
];

const DEFAULT_INTERVAL: &str = "1h";
const DEFAULT_MAX_PASSES: usize = 3;
const DEFAULT_DISCOVERY_START: &str = "2019-01-01";
const DISCOVERY_DAILY_CHUNK_DAYS: i64 = 365;
const FILE_MAGIC: u32 = 0x43444C33; // "CDL3"
const HEADER_SIZE: usize = 12;
const RECORD_SIZE: usize = 64;
const FOOTER_SIZE: usize = 4;

fn crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

#[derive(Debug)]
struct SymbolRepairReport {
    first_exchange_open: DateTime<Utc>,
    gaps_before: usize,
    fetched_candles: usize,
    unresolved_gaps: Vec<(DateTime<Utc>, DateTime<Utc>)>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut end_str: Option<String> = None;
    let mut symbols_str: Option<String> = None;
    let mut symbols_from_store = false;
    let mut interval = DEFAULT_INTERVAL.to_string();
    let mut search_start = parse_datetime_or_date(DEFAULT_DISCOVERY_START);
    let mut max_passes = DEFAULT_MAX_PASSES;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--end" | "-e" => {
                i += 1;
                end_str = args.get(i).cloned();
            }
            "--symbols" => {
                i += 1;
                symbols_str = args.get(i).cloned();
            }
            "--symbols-from-store" => {
                symbols_from_store = true;
            }
            "--interval" => {
                i += 1;
                interval = args.get(i).cloned().unwrap_or_else(|| interval.clone());
            }
            "--search-start" => {
                i += 1;
                search_start = parse_datetime_or_date(
                    args.get(i)
                        .map(String::as_str)
                        .unwrap_or(DEFAULT_DISCOVERY_START),
                );
            }
            "--max-passes" => {
                i += 1;
                max_passes = args
                    .get(i)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(DEFAULT_MAX_PASSES);
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                print_usage();
                process::exit(1);
            }
        }
        i += 1;
    }

    let end = end_str
        .as_deref()
        .map(parse_datetime_or_date)
        .unwrap_or_else(|| {
            eprintln!("Error: --end is required");
            print_usage();
            process::exit(1);
        });

    let step = interval_duration(&interval).unwrap_or_else(|| {
        eprintln!("Unsupported interval: {interval}");
        process::exit(1);
    });

    let aligned_end = align_to_interval_boundary(end, step);
    if aligned_end <= search_start {
        eprintln!(
            "Error: aligned end {} must be after search start {}",
            aligned_end.format("%Y-%m-%d %H:%M"),
            search_start.format("%Y-%m-%d %H:%M"),
        );
        process::exit(1);
    }

    let store_dir = candle_store_dir();
    let symbols: Vec<String> = if symbols_from_store {
        discover_symbols_from_store(&store_dir, &interval)
    } else {
        match symbols_str {
            Some(raw) => raw.split(',').map(|s| s.trim().to_uppercase()).collect(),
            None => DEFAULT_SYMBOLS.iter().map(|s| s.to_string()).collect(),
        }
    };

    if symbols.is_empty() {
        eprintln!("No symbols found for interval {interval}");
        process::exit(1);
    }

    let client = BinanceClient::new(MarketType::Futures);
    let mut store = CandleStore::new();

    println!("=== ct-repair-gaps ===");
    println!("  Interval:       {interval}");
    println!(
        "  Search start:   {}",
        search_start.format("%Y-%m-%d %H:%M")
    );
    println!("  Requested end:  {}", end.format("%Y-%m-%d %H:%M"));
    println!("  Aligned end:    {}", aligned_end.format("%Y-%m-%d %H:%M"));
    println!(
        "  Symbols:        {} ({})",
        symbols.len(),
        symbols.join(", ")
    );
    println!("  Max passes:     {max_passes}");
    println!("  Discovery span: {DISCOVERY_DAILY_CHUNK_DAYS} day daily chunks");
    println!();

    let t_total = Instant::now();
    let mut reports = Vec::new();
    let mut missing_on_exchange = Vec::new();
    let mut failed_symbols = Vec::new();

    for sym in &symbols {
        println!("== {sym} ==");
        let t_symbol = Instant::now();
        match repair_symbol(
            &mut store,
            &client,
            sym,
            &interval,
            search_start,
            aligned_end,
            max_passes,
        ) {
            Ok(Some(report)) => {
                println!(
                    "  first exchange candle: {}",
                    report.first_exchange_open.format("%Y-%m-%d %H:%M")
                );
                println!("  gaps before repair:   {}", report.gaps_before);
                println!("  candles fetched:      {}", report.fetched_candles);
                if report.unresolved_gaps.is_empty() {
                    println!("  unresolved gaps:      none");
                } else {
                    println!("  unresolved gaps:      {}", report.unresolved_gaps.len());
                    for (start, end) in &report.unresolved_gaps {
                        println!(
                            "    [{} -> {})",
                            start.format("%Y-%m-%d %H:%M"),
                            end.format("%Y-%m-%d %H:%M"),
                        );
                    }
                }
                println!(
                    "  completed in {:.0}ms",
                    t_symbol.elapsed().as_secs_f64() * 1000.0
                );
                println!();
                reports.push(report);
            }
            Ok(None) => {
                println!("  no exchange candles found before requested end");
                println!(
                    "  completed in {:.0}ms",
                    t_symbol.elapsed().as_secs_f64() * 1000.0
                );
                println!();
                missing_on_exchange.push(sym.clone());
            }
            Err(err) => {
                eprintln!("  ERROR: {err}");
                eprintln!(
                    "  failed after {:.0}ms",
                    t_symbol.elapsed().as_secs_f64() * 1000.0
                );
                eprintln!();
                failed_symbols.push(sym.clone());
            }
        }
    }

    let repaired_symbols = reports
        .iter()
        .filter(|r| r.gaps_before > 0 && r.unresolved_gaps.is_empty())
        .count();
    let symbols_with_unresolved = reports
        .iter()
        .filter(|r| !r.unresolved_gaps.is_empty())
        .count();
    let total_fetched: usize = reports.iter().map(|r| r.fetched_candles).sum();

    println!("=== Summary ===");
    println!("  symbols processed:   {}", reports.len());
    println!("  symbols repaired:    {repaired_symbols}");
    println!("  unresolved symbols:  {symbols_with_unresolved}");
    println!("  candles fetched:     {total_fetched}");
    if !missing_on_exchange.is_empty() {
        println!(
            "  no exchange data:    {} ({})",
            missing_on_exchange.len(),
            missing_on_exchange.join(", "),
        );
    }
    if !failed_symbols.is_empty() {
        println!(
            "  failed symbols:      {} ({})",
            failed_symbols.len(),
            failed_symbols.join(", "),
        );
    }
    println!(
        "  total runtime:       {:.0}ms",
        t_total.elapsed().as_secs_f64() * 1000.0
    );

    if !failed_symbols.is_empty() {
        process::exit(1);
    }
}

fn repair_symbol(
    store: &mut CandleStore,
    client: &BinanceClient,
    symbol: &str,
    interval: &str,
    search_start: DateTime<Utc>,
    end: DateTime<Utc>,
    max_passes: usize,
) -> Result<Option<SymbolRepairReport>, String> {
    let step =
        interval_duration(interval).ok_or_else(|| format!("unsupported interval {interval}"))?;
    let path = candle_store_file(symbol, interval);
    let local_first_open = read_first_local_open(&path)?;
    let discovery_end = local_first_open
        .map(|ts| (start_of_day(ts) + Duration::days(1)).min(end))
        .unwrap_or(end);
    let first_exchange_open =
        match find_first_exchange_open(client, symbol, interval, search_start, discovery_end)? {
            Some(ts) => ts,
            None => return Ok(None),
        };

    let mut total_fetched = 0usize;
    let mut gaps_before = 0usize;

    for pass in 1..=max_passes {
        let gaps = scan_local_gap_ranges(&path, first_exchange_open, end, step)?;
        if pass == 1 {
            gaps_before = gaps.len();
        }
        if gaps.is_empty() {
            break;
        }

        println!("  pass {pass}/{max_passes}: {} gap range(s)", gaps.len());
        let mut fetched_this_pass = 0usize;

        for (gap_start, gap_end) in &gaps {
            println!(
                "    fetching [{} -> {})",
                gap_start.format("%Y-%m-%d %H:%M"),
                gap_end.format("%Y-%m-%d %H:%M"),
            );
            let candles = fetch_range_fresh(client, symbol, interval, *gap_start, *gap_end)?;
            if candles.is_empty() {
                println!("      no candles returned");
                continue;
            }
            fetched_this_pass += candles.len();
            total_fetched += candles.len();
            store.put(symbol, interval, &candles);
            println!("      merged {} candle(s)", candles.len());
        }

        if fetched_this_pass == 0 {
            break;
        }
    }

    let unresolved_gaps = scan_local_gap_ranges(&path, first_exchange_open, end, step)?;

    Ok(Some(SymbolRepairReport {
        first_exchange_open,
        gaps_before,
        fetched_candles: total_fetched,
        unresolved_gaps,
    }))
}

fn find_first_exchange_open(
    client: &BinanceClient,
    symbol: &str,
    interval: &str,
    search_start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, String> {
    let mut chunk_start = start_of_day(search_start);

    while chunk_start < end {
        let chunk_end = (chunk_start + Duration::days(DISCOVERY_DAILY_CHUNK_DAYS)).min(end);
        let chunk = fetch_range_fresh(client, symbol, "1d", chunk_start, chunk_end)?;
        if chunk.is_empty() {
            chunk_start = chunk_end;
            continue;
        }

        let first_day_open = chunk.first().unwrap().open_time;
        if interval == "1d" {
            return Ok(Some(first_day_open));
        }

        let exact_end = (first_day_open + Duration::days(1)).min(end);
        let exact = fetch_range_fresh(client, symbol, interval, first_day_open, exact_end)?;
        if let Some(first) = exact.first() {
            return Ok(Some(first.open_time));
        }

        return Err(format!(
            "discovery inconsistency for {symbol}: 1d data exists on {} but {interval} data is empty within that day",
            first_day_open.format("%Y-%m-%d"),
        ));
    }

    Ok(None)
}

fn fetch_range_fresh(
    client: &BinanceClient,
    symbol: &str,
    interval: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Candle>, String> {
    if start >= end {
        return Ok(Vec::new());
    }
    // The parsed kline cache was removed; individual pages are cached by URL
    // in get_json's mem+disk caches. A retry after partial failure reuses
    // succeeded pages and re-fetches only the missing ones — no explicit
    // invalidation needed here.
    client
        .fetch_klines(symbol, interval, start, end)
        .map(|kf| kf.into_rows())
        .map_err(|e| format!("{interval} fetch failed for {symbol}: {e}"))
}

fn scan_missing_ranges(
    open_times: impl IntoIterator<Item = DateTime<Utc>>,
    expected_start: DateTime<Utc>,
    end: DateTime<Utc>,
    step: Duration,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let step_ms = step.num_milliseconds();
    let mut cursor_ms = expected_start.timestamp_millis();
    let end_ms = end.timestamp_millis();
    let mut gaps = Vec::new();

    for open_time in open_times {
        let open_ms = open_time.timestamp_millis();
        if open_ms > cursor_ms {
            gaps.push((ms_to_dt(cursor_ms), open_time));
        }
        let next_cursor_ms = open_ms + step_ms;
        if next_cursor_ms > cursor_ms {
            cursor_ms = next_cursor_ms;
        }
    }

    if cursor_ms < end_ms {
        gaps.push((ms_to_dt(cursor_ms), end));
    }

    gaps
}

fn interval_duration(interval: &str) -> Option<Duration> {
    match interval {
        "1m" => Some(Duration::minutes(1)),
        "3m" => Some(Duration::minutes(3)),
        "5m" => Some(Duration::minutes(5)),
        "15m" => Some(Duration::minutes(15)),
        "30m" => Some(Duration::minutes(30)),
        "1h" => Some(Duration::hours(1)),
        "2h" => Some(Duration::hours(2)),
        "4h" => Some(Duration::hours(4)),
        "6h" => Some(Duration::hours(6)),
        "8h" => Some(Duration::hours(8)),
        "12h" => Some(Duration::hours(12)),
        "1d" => Some(Duration::days(1)),
        _ => None,
    }
}

fn align_to_interval_boundary(dt: DateTime<Utc>, step: Duration) -> DateTime<Utc> {
    let step_ms = step.num_milliseconds();
    let aligned_ms = dt.timestamp_millis().div_euclid(step_ms) * step_ms;
    ms_to_dt(aligned_ms)
}

fn start_of_day(dt: DateTime<Utc>) -> DateTime<Utc> {
    let d = dt.date_naive();
    Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
}


fn parse_datetime_or_date(input: &str) -> DateTime<Utc> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return dt.with_timezone(&Utc);
    }

    let date = NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap_or_else(|_| {
        eprintln!("Invalid date/datetime: {input}");
        process::exit(1);
    });
    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
}

fn candle_store_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join(".claude_trader")
        .join("candle_store")
}

fn candle_store_file(symbol: &str, interval: &str) -> PathBuf {
    candle_store_dir().join(format!("{symbol}_{interval}.bin"))
}

fn discover_symbols_from_store(store_dir: &Path, interval: &str) -> Vec<String> {
    let suffix = format!("_{interval}.bin");
    let mut out = Vec::new();

    if let Ok(entries) = fs::read_dir(store_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if !name.ends_with(&suffix) {
                continue;
            }
            let Some(symbol) = name.strip_suffix(&suffix) else {
                continue;
            };
            if is_likely_trade_symbol(symbol) {
                out.push(symbol.to_string());
            }
        }
    }

    out.sort();
    out.dedup();
    out
}

fn is_likely_trade_symbol(symbol: &str) -> bool {
    symbol.ends_with("USDT")
        && symbol
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn read_first_local_open(path: &Path) -> Result<Option<DateTime<Utc>>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("failed to read {}: {err}", path.display())),
    };

    let available = validated_record_count(&bytes, path)?;
    if available == 0 {
        return Ok(None);
    }

    let data = &bytes[HEADER_SIZE..];
    let first_open_ms = i64::from_le_bytes(data[0..8].try_into().unwrap());
    Ok(Some(ms_to_dt(first_open_ms)))
}

fn scan_local_gap_ranges(
    path: &Path,
    expected_start: DateTime<Utc>,
    end: DateTime<Utc>,
    step: Duration,
) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(if expected_start < end {
                vec![(expected_start, end)]
            } else {
                Vec::new()
            })
        }
        Err(err) => return Err(format!("failed to read {}: {err}", path.display())),
    };

    let available = validated_record_count(&bytes, path)?;
    if available == 0 {
        return Ok(if expected_start < end {
            vec![(expected_start, end)]
        } else {
            Vec::new()
        });
    }

    let data = &bytes[HEADER_SIZE..];
    let open_times = (0..available)
        .filter_map(|i| {
            let offset = i * RECORD_SIZE;
            let open_ms = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            let open_time = ms_to_dt(open_ms);
            (open_time >= expected_start && open_time < end).then_some(open_time)
        })
        .collect::<Vec<_>>();

    Ok(scan_missing_ranges(open_times, expected_start, end, step))
}

fn validated_record_count(bytes: &[u8], path: &Path) -> Result<usize, String> {
    if bytes.len() < HEADER_SIZE + FOOTER_SIZE {
        return Ok(0);
    }

    // Verify file-level CRC (covers magic + count + records)
    let content = &bytes[..bytes.len() - FOOTER_SIZE];
    let stored_crc = u32::from_le_bytes(bytes[bytes.len() - FOOTER_SIZE..].try_into().unwrap());
    if crc32(content) != stored_crc {
        eprintln!(
            "WARNING: {} failed CRC check — deleting corrupt file.",
            path.display()
        );
        fs::remove_file(path).ok();
        return Ok(0);
    }

    let magic = u32::from_le_bytes(content[0..4].try_into().unwrap());
    if magic != FILE_MAGIC {
        return Err(format!(
            "unexpected candle file magic in {}",
            path.display()
        ));
    }
    let count = u64::from_le_bytes(content[4..12].try_into().unwrap()) as usize;
    let payload = &content[HEADER_SIZE..];
    Ok((payload.len() / RECORD_SIZE).min(count))
}

fn print_usage() {
    eprintln!(
        "Usage: ct-repair-gaps --end <YYYY-MM-DD|RFC3339> [options]\n\
         \n\
         Options:\n\
           --symbols BTCUSDT,ETHUSDT   Comma-separated symbol list\n\
           --symbols-from-store        Discover symbols from local candle_store for the interval\n\
           --interval 1h               Candle interval to repair (default: 1h)\n\
           --search-start 2019-01-01   Discovery lower bound for first exchange candle\n\
           --max-passes 3              Maximum repair passes per symbol\n\
           --help                      Show this help\n\
         \n\
         Notes:\n\
           - End is treated as an exclusive boundary and aligned down to the interval.\n\
           - The tool discovers the first exchange candle, scans continuity from there,\n\
             fetches missing ranges, merges them into CandleStore, and reports any gaps\n\
             that remain after repair passes."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_missing_ranges_finds_internal_and_tail_gaps() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let candles = vec![
            candle_at(start),
            candle_at(start + Duration::hours(1)),
            candle_at(start + Duration::hours(4)),
        ];

        let gaps = scan_missing_ranges(
            candles.into_iter().map(|c| c.open_time),
            start,
            start + Duration::hours(6),
            Duration::hours(1),
        );

        assert_eq!(
            gaps,
            vec![
                (start + Duration::hours(2), start + Duration::hours(4)),
                (start + Duration::hours(5), start + Duration::hours(6)),
            ],
        );
    }

    #[test]
    fn align_to_interval_boundary_floors_end() {
        let dt = Utc.with_ymd_and_hms(2026, 1, 3, 18, 37, 0).unwrap();
        let aligned = align_to_interval_boundary(dt, Duration::hours(1));
        assert_eq!(aligned, Utc.with_ymd_and_hms(2026, 1, 3, 18, 0, 0).unwrap());
    }

    fn candle_at(open_time: DateTime<Utc>) -> Candle {
        Candle {
            open_time,
            close_time: open_time + Duration::hours(1),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
            taker_buy_volume: 1.0,
        }
    }
}
