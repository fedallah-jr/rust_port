//! Window grouping and period management.

use chrono::{DateTime, Duration, Utc};
use claude_trader_models::EvalWindow;

/// A contiguous fetch period merging multiple evaluation windows.
#[derive(Debug, Clone)]
pub struct FetchPeriod {
    pub windows: Vec<EvalWindow>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Group sorted windows into contiguous fetch periods.
///
/// Windows with a gap smaller than `gap_threshold` are merged into the same
/// period to avoid redundant data fetches.
pub fn group_into_periods(windows: &[EvalWindow], gap_threshold: Duration) -> Vec<FetchPeriod> {
    if windows.is_empty() {
        return Vec::new();
    }

    let mut sorted: Vec<&EvalWindow> = windows.iter().collect();
    sorted.sort_by_key(|w| w.start);

    let mut periods = Vec::new();
    let mut current_windows = vec![sorted[0].clone()];
    let mut current_end = sorted[0].end;

    for &w in &sorted[1..] {
        if w.start - current_end <= gap_threshold {
            current_windows.push(w.clone());
            if w.end > current_end {
                current_end = w.end;
            }
        } else {
            let start = current_windows.first().unwrap().start;
            periods.push(FetchPeriod {
                start,
                end: current_end,
                windows: current_windows,
            });
            current_windows = vec![w.clone()];
            current_end = w.end;
        }
    }

    let start = current_windows.first().unwrap().start;
    periods.push(FetchPeriod {
        start,
        end: current_end,
        windows: current_windows,
    });

    periods
}
