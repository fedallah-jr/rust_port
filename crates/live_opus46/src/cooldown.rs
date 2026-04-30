//! Durable cooldown enforcement for the live opus46 adapter.
//!
//! The research runtime authoritatively enforces `CooldownSpec` across every
//! signal in a run via `enforce_signal_cooldown`. Live trading needs the
//! same gate, but applied incrementally as new signals arrive — and it must
//! survive process restarts so a crash 30 minutes after emitting a 6h-cooldown
//! signal doesn't re-emit it on relaunch.
//!
//! Implementation:
//!   - In-memory `last_seen: HashMap<CooldownKey-as-str, DateTime<Utc>>`.
//!   - Persisted as JSON at `state_path` (atomic tmp+rename mirroring the
//!     position tracker's pattern).
//!   - Stale entries are kept until their cooldown elapses naturally; we
//!     don't aggressively prune because key strings are tiny.
//!
//! `CooldownSpec.hours == 0` means "no cooldown" — `record` is a no-op and
//! `is_blocked` always returns false. Matches the runtime semantic in
//! `crates/evaluator/src/cooldown.rs`.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use claude_trader_models::CooldownSpec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CooldownStore {
    /// Maps `CooldownKey::as_str()` → most recent emit time. Persisted as
    /// RFC3339 strings via chrono's serde feature so the file is
    /// human-inspectable.
    last_seen: HashMap<String, DateTime<Utc>>,
}

impl CooldownStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from disk. Best-effort: missing file → empty store; malformed
    /// file → log to stderr and start fresh (matches the position tracker's
    /// load_state behavior for resilience).
    pub fn load(path: &Path) -> Self {
        let raw = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self::new(),
            Err(e) => {
                eprintln!("cooldown load {}: {e}", path.display());
                return Self::new();
            }
        };
        match serde_json::from_str::<Self>(&raw) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("cooldown parse {}: {e}", path.display());
                Self::new()
            }
        }
    }

    /// Atomic tmp+rename save. Returns Ok on success or Err with the
    /// underlying io error so callers can decide how to report.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let pid = std::process::id();
        let tmp = path.with_file_name(format!(
            "{}.{}.tmp",
            path.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "cooldown.json".to_string()),
            pid,
        ));
        let payload = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&payload)?;
        f.sync_all()?;
        drop(f);
        if let Err(e) = fs::rename(&tmp, path) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
        Ok(())
    }

    /// Returns true iff `signal_date` is within `spec.hours` of the most
    /// recent recorded emit for `spec.key`. Returns false when `spec.hours`
    /// is zero (no cooldown).
    pub fn is_blocked(&self, spec: &CooldownSpec, signal_date: DateTime<Utc>) -> bool {
        if spec.hours <= 0.0 {
            return false;
        }
        match self.last_seen.get(spec.key.as_str()) {
            None => false,
            Some(prev) => {
                let cooldown_secs = (spec.hours * 3600.0).round() as i64;
                (signal_date - *prev).num_seconds() < cooldown_secs
            }
        }
    }

    /// Record an emit. Updates only when `signal_date` advances the existing
    /// entry — out-of-order replays don't move the gate backward.
    pub fn record(&mut self, spec: &CooldownSpec, signal_date: DateTime<Utc>) {
        if spec.hours <= 0.0 {
            return;
        }
        let entry = self
            .last_seen
            .entry(spec.key.as_str().to_string())
            .or_insert(signal_date);
        if signal_date > *entry {
            *entry = signal_date;
        }
    }

    /// Default path: `$HOME/.claude_trader/live_cooldown_<strategy_id>.json`.
    pub fn default_path(strategy_id: &str) -> PathBuf {
        let home = std::env::var("HOME")
            .ok()
            .or_else(|| std::env::var("USERPROFILE").ok())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let safe_id: String = strategy_id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        home.join(".claude_trader")
            .join(format!("live_cooldown_{safe_id}.json"))
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.last_seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use claude_trader_models::{CooldownKey, CooldownSpec};

    fn spec(key: &str, hours: f64) -> CooldownSpec {
        CooldownSpec {
            key: CooldownKey::custom(key),
            hours,
        }
    }

    #[test]
    fn unset_keys_never_block() {
        let s = CooldownStore::new();
        let sp = spec("x", 6.0);
        assert!(!s.is_blocked(&sp, Utc::now()));
    }

    #[test]
    fn record_then_block_within_window() {
        let mut s = CooldownStore::new();
        let sp = spec("x", 6.0);
        let t0 = Utc::now();
        s.record(&sp, t0);
        assert!(s.is_blocked(&sp, t0 + Duration::hours(1)));
        assert!(s.is_blocked(&sp, t0 + Duration::hours(5) + Duration::minutes(59)));
    }

    #[test]
    fn unblocks_after_window_elapses() {
        let mut s = CooldownStore::new();
        let sp = spec("x", 6.0);
        let t0 = Utc::now();
        s.record(&sp, t0);
        assert!(!s.is_blocked(&sp, t0 + Duration::hours(6)));
        assert!(!s.is_blocked(&sp, t0 + Duration::hours(7)));
    }

    #[test]
    fn zero_hours_is_no_op() {
        let mut s = CooldownStore::new();
        let sp = spec("x", 0.0);
        s.record(&sp, Utc::now());
        assert_eq!(s.entry_count(), 0);
        assert!(!s.is_blocked(&sp, Utc::now()));
    }

    #[test]
    fn out_of_order_record_does_not_regress() {
        let mut s = CooldownStore::new();
        let sp = spec("x", 6.0);
        let later = Utc::now();
        let earlier = later - Duration::hours(1);
        s.record(&sp, later);
        s.record(&sp, earlier);
        assert!(s.is_blocked(&sp, later + Duration::hours(1)));
    }

    #[test]
    fn round_trip_through_disk() {
        let dir = tempdir_path();
        let path = dir.join("cd.json");
        let mut a = CooldownStore::new();
        let sp = spec("BTC|short", 6.0);
        let t0 = Utc::now();
        a.record(&sp, t0);
        a.save(&path).expect("save");

        let b = CooldownStore::load(&path);
        assert!(b.is_blocked(&sp, t0 + Duration::hours(2)));
        assert!(!b.is_blocked(&sp, t0 + Duration::hours(7)));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_loads_empty() {
        let dir = tempdir_path();
        let path = dir.join("does_not_exist.json");
        let s = CooldownStore::load(&path);
        assert_eq!(s.entry_count(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_path_sanitizes_strategy_id() {
        let p = CooldownStore::default_path("abc/xyz?!");
        assert!(p.to_string_lossy().contains("live_cooldown_abc_xyz__"));
    }

    fn tempdir_path() -> PathBuf {
        let pid = std::process::id();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("opus46_cd_test_{pid}_{nonce}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
