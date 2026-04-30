//! Cooldown enforcement — filters signals by per-signal `CooldownSpec`.
//!
//! The runtime is the single authority for cooldown. Strategies emit every
//! candidate signal and return a `CooldownSpec` per signal; this filter
//! applies globally across a run.
//!
//! Hours are normalized to integer seconds (rounded) at enforcement time so
//! that NaN/inf are rejected up front and float drift cannot produce inconsistent
//! comparisons for keys that would otherwise be identical.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use claude_trader_models::{CooldownKey, CooldownSpec, Signal};

/// Enforce cooldown on raw signals. Input order is preserved.
///
/// For each signal, `spec_fn` is invoked to obtain a `CooldownSpec`. The
/// function panics if a spec returns non-finite or negative hours. Under
/// `debug_assertions`, it also panics when two signals share a key but
/// disagree on the cooldown duration.
pub fn enforce_signal_cooldown<F>(signals: &[Arc<Signal>], spec_fn: F) -> Vec<Arc<Signal>>
where
    F: Fn(&Signal) -> CooldownSpec,
{
    let mut last_seen: HashMap<CooldownKey, (DateTime<Utc>, i64)> = HashMap::new();
    let mut out = Vec::with_capacity(signals.len());

    for sig in signals {
        let spec = spec_fn(sig);
        assert!(
            spec.hours.is_finite() && spec.hours >= 0.0,
            "cooldown hours must be finite and non-negative (got {h} for key {k:?})",
            h = spec.hours,
            k = spec.key,
        );

        if spec.hours == 0.0 {
            out.push(Arc::clone(sig));
            continue;
        }

        let cooldown_secs = (spec.hours * 3600.0).round() as i64;
        let keep = match last_seen.get(&spec.key) {
            None => true,
            Some(&(last_t, prev_secs)) => {
                debug_assert!(
                    prev_secs == cooldown_secs,
                    "cooldown hours inconsistent for key {k:?}: previous={prev}s current={cur}s",
                    k = spec.key,
                    prev = prev_secs,
                    cur = cooldown_secs,
                );
                (sig.signal_date - last_t).num_seconds() >= cooldown_secs
            }
        };

        if keep {
            last_seen.insert(spec.key, (sig.signal_date, cooldown_secs));
            out.push(Arc::clone(sig));
        }
    }

    out
}
