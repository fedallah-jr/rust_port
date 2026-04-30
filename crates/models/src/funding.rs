//! Funding rate analysis utilities.
//!
//! Helpers for computing funding rate context metrics from a sorted
//! `Vec<FundingRate>`. All functions expect rates sorted by timestamp
//! (ascending).

use chrono::{DateTime, Duration, Utc};

use crate::FundingRate;

/// Snapshot of funding rate context at a point in time.
#[derive(Debug, Clone, Copy)]
pub struct FundingContext {
    /// Current funding rate as a z-score against its 30-day rolling
    /// distribution. Normalizes "extreme" across symbols and regimes.
    /// Positive = funding is elevated relative to recent history.
    pub zscore_30d: f64,

    /// Sum of funding rates over the last 7 days (~21 periods at 8h
    /// frequency). Captures sustained directional carry pressure.
    pub cumulative_7d: f64,

    /// Difference between the most recent funding rate and the one
    /// before it. Positive = funding is accelerating upward.
    pub rate_change: f64,

    /// The raw funding rate at this time.
    pub rate: f64,
}

/// Compute funding context at time `t` from a sorted slice of funding rates.
///
/// Returns `None` if there are fewer than 3 rates before `t` (insufficient
/// history for meaningful statistics).
///
/// # Arguments
/// * `rates` — Funding rates sorted by timestamp ascending.
/// * `t` — The point in time to evaluate. Uses the most recent rate at or
///   before `t`.
pub fn funding_context_at(rates: &[FundingRate], t: DateTime<Utc>) -> Option<FundingContext> {
    // Find the most recent rate at or before t
    let idx = rates.partition_point(|r| r.timestamp <= t);
    if idx < 3 {
        return None;
    }
    let current_idx = idx - 1;
    let current_rate = rates[current_idx].funding_rate;
    let prev_rate = rates[current_idx - 1].funding_rate;

    // rate_change: difference from previous period
    let rate_change = current_rate - prev_rate;

    // cumulative_7d: sum of rates in last 7 days
    let cutoff_7d = t - Duration::days(7);
    let start_7d = rates.partition_point(|r| r.timestamp < cutoff_7d);
    let cumulative_7d: f64 = rates[start_7d..idx].iter().map(|r| r.funding_rate).sum();

    // zscore_30d: z-score against 30-day distribution
    let cutoff_30d = t - Duration::days(30);
    let start_30d = rates.partition_point(|r| r.timestamp < cutoff_30d);
    let window = &rates[start_30d..idx];

    if window.len() < 3 {
        return Some(FundingContext {
            zscore_30d: 0.0,
            cumulative_7d,
            rate_change,
            rate: current_rate,
        });
    }

    let n = window.len() as f64;
    let mean: f64 = window.iter().map(|r| r.funding_rate).sum::<f64>() / n;
    let variance: f64 =
        window.iter().map(|r| (r.funding_rate - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    let zscore_30d = if std_dev > 1e-12 {
        (current_rate - mean) / std_dev
    } else {
        0.0
    };

    Some(FundingContext {
        zscore_30d,
        cumulative_7d,
        rate_change,
        rate: current_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_rates(values: &[(i64, f64)]) -> Vec<FundingRate> {
        values
            .iter()
            .map(|&(hours, rate)| FundingRate {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
                    + Duration::hours(hours),
                funding_rate: rate,
                mark_price: None,
            })
            .collect()
    }

    #[test]
    fn test_basic_context() {
        // 8h-spaced rates over ~3 days
        let rates = make_rates(&[
            (0, 0.0001),
            (8, 0.0002),
            (16, 0.0001),
            (24, 0.0003),
            (32, 0.0001),
            (40, 0.0002),
            (48, 0.0005),
            (56, 0.0001),
            (64, 0.0004),
            (72, 0.0010), // spike
        ]);

        let t = Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap();
        let ctx = funding_context_at(&rates, t).unwrap();

        assert_eq!(ctx.rate, 0.0010);
        assert!((ctx.rate_change - 0.0006).abs() < 1e-10); // 0.0010 - 0.0004
        assert!(ctx.zscore_30d > 1.0); // spike should be elevated
        assert!(ctx.cumulative_7d > 0.0);
    }

    #[test]
    fn test_insufficient_data() {
        let rates = make_rates(&[(0, 0.0001), (8, 0.0002)]);
        let t = Utc.with_ymd_and_hms(2024, 1, 1, 16, 0, 0).unwrap();
        assert!(funding_context_at(&rates, t).is_none());
    }

}
