//! `TwoStageBook<S>` — a setup → trigger state machine.
//!
//! A lot of price-action strategies want to: (1) identify a "setup"
//! bar where a pattern completes, (2) wait a bounded number of bars for
//! a subsequent "trigger" bar that confirms the setup, (3) consume the
//! setup on first confirmation.
//!
//! The naive Vec-plus-bool-consumed-flag pattern used in opus47_2 / 3 is
//! correct but copy-pasted everywhere. `TwoStageBook` generalizes it:
//!
//! ```ignore
//! let mut book = TwoStageBook::<MySetup>::new(8); // TTL = 8 bars
//! for i in start_idx..n {
//!     if let Some(setup) = detect_setup(i) {
//!         book.push(i, setup);
//!     }
//!     if let Some((setup_idx, setup)) = book.consume_latest(i, |_, s| {
//!         trigger_condition_met(i, s)
//!     }) {
//!         emit_signal_at(i, setup_idx, setup);
//!     }
//! }
//! ```
//!
//! ## Purity
//!
//! `TwoStageBook` owns no state beyond its internal Vec. It must be
//! constructed fresh inside each `generate_signals()` call. No caching,
//! no thread-locals, no interior mutability beyond the expected Vec
//! append + in-place flag flip.
//!
//! ## Performance
//!
//! - `push`: amortized O(1).
//! - `consume_latest`: O(outstanding) worst case, where "outstanding"
//!   is at most `ttl` bars' worth of setups (typically ≤ 8).
//! - No allocation per call after the initial capacity is reached.

use std::fmt;

#[derive(Clone, Copy)]
struct Slot<S> {
    bar_idx: usize,
    setup: S,
    consumed: bool,
}

/// A generic setup → trigger book with a TTL (measured in bar indices).
///
/// `S` is the setup payload — typically a small `Copy` struct like
/// `DivergenceDetection` plus whatever per-bar state the trigger needs
/// (e.g. the setup bar's low for a swing-low break).
pub struct TwoStageBook<S> {
    slots: Vec<Slot<S>>,
    ttl: usize,
    /// Pointer to the oldest slot whose TTL has NOT yet expired as of
    /// the last `consume_latest` call. Everything before this index is
    /// guaranteed expired (or consumed) and can be skipped in future
    /// iterations. Monotonically non-decreasing.
    oldest_live: usize,
}

impl<S> TwoStageBook<S> {
    pub fn new(ttl: usize) -> Self {
        Self {
            slots: Vec::new(),
            ttl,
            oldest_live: 0,
        }
    }

    /// Pre-allocate capacity. Optional; purely a perf hint for strategies
    /// with known-bounded setup counts per call.
    pub fn with_capacity(ttl: usize, capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            ttl,
            oldest_live: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Record a new setup at bar index `bar_idx`. Setups must be pushed
    /// in non-decreasing `bar_idx` order (the usual forward-scan pattern).
    #[inline]
    pub fn push(&mut self, bar_idx: usize, setup: S) {
        debug_assert!(
            self.slots
                .last()
                .map_or(true, |s| s.bar_idx <= bar_idx),
            "TwoStageBook requires monotonically non-decreasing bar_idx"
        );
        self.slots.push(Slot {
            bar_idx,
            setup,
            consumed: false,
        });
    }
}

impl<S: Copy> TwoStageBook<S> {
    /// Scan from the most recent setup backwards and consume the first
    /// non-consumed slot where:
    ///
    /// - `now_idx > bar_idx` (setup must be in the past)
    /// - `now_idx - bar_idx <= ttl` (not expired)
    /// - `pred(bar_idx, &setup)` returns `true`
    ///
    /// On match, marks the slot consumed and returns `(bar_idx, setup)`.
    /// Youngest-first means if multiple setups stack, the most recent
    /// one wins — matching the behaviour of opus47_2 v25's inline scan.
    pub fn consume_latest<F>(&mut self, now_idx: usize, pred: F) -> Option<(usize, S)>
    where
        F: Fn(usize, &S) -> bool,
    {
        // Advance the pruning pointer past anything fully expired.
        let cutoff = now_idx.saturating_sub(self.ttl);
        while self.oldest_live < self.slots.len()
            && self.slots[self.oldest_live].bar_idx < cutoff
        {
            self.oldest_live += 1;
        }

        // Iterate from the newest slot back to `oldest_live`.
        for idx in (self.oldest_live..self.slots.len()).rev() {
            let slot = &self.slots[idx];
            if slot.consumed {
                continue;
            }
            if slot.bar_idx >= now_idx {
                continue;
            }
            if now_idx - slot.bar_idx > self.ttl {
                // This slot is expired; everything older is too, but we
                // can't advance `oldest_live` here because later
                // iterations will detect them on the next call.
                continue;
            }
            if !pred(slot.bar_idx, &slot.setup) {
                continue;
            }
            // Match. Mark consumed and return.
            let matched_idx = slot.bar_idx;
            let matched_setup = slot.setup;
            self.slots[idx].consumed = true;
            return Some((matched_idx, matched_setup));
        }
        None
    }
}

impl<S: fmt::Debug> fmt::Debug for TwoStageBook<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TwoStageBook")
            .field("ttl", &self.ttl)
            .field("slots", &self.slots.len())
            .field("oldest_live", &self.oldest_live)
            .finish()
    }
}

impl<S: fmt::Debug> fmt::Debug for Slot<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slot")
            .field("bar_idx", &self.bar_idx)
            .field("consumed", &self.consumed)
            .field("setup", &self.setup)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct TestSetup {
        trigger_level: f64,
    }

    #[test]
    fn consume_matches_youngest_first() {
        let mut book = TwoStageBook::<TestSetup>::new(8);
        book.push(10, TestSetup { trigger_level: 100.0 });
        book.push(12, TestSetup { trigger_level: 101.0 });
        // At bar 14, both setups are active. Predicate matches both.
        let (idx, setup) = book.consume_latest(14, |_, _| true).unwrap();
        assert_eq!(idx, 12);
        assert_eq!(setup.trigger_level, 101.0);
    }

    #[test]
    fn consume_skips_already_consumed() {
        let mut book = TwoStageBook::<TestSetup>::new(8);
        book.push(10, TestSetup { trigger_level: 100.0 });
        book.push(12, TestSetup { trigger_level: 101.0 });
        let (idx1, _) = book.consume_latest(14, |_, _| true).unwrap();
        assert_eq!(idx1, 12);
        let (idx2, _) = book.consume_latest(14, |_, _| true).unwrap();
        assert_eq!(idx2, 10);
        assert!(book.consume_latest(14, |_, _| true).is_none());
    }

    #[test]
    fn consume_respects_ttl() {
        let mut book = TwoStageBook::<TestSetup>::new(3);
        book.push(10, TestSetup { trigger_level: 100.0 });
        // TTL 3, query at bar 15 -> 10 is expired (15-10=5 > 3).
        assert!(book.consume_latest(15, |_, _| true).is_none());
    }

    #[test]
    fn consume_skips_setup_at_same_bar() {
        let mut book = TwoStageBook::<TestSetup>::new(8);
        book.push(10, TestSetup { trigger_level: 100.0 });
        // At bar 10 itself, no consumption (setup must be in the past).
        assert!(book.consume_latest(10, |_, _| true).is_none());
    }

    #[test]
    fn consume_applies_predicate() {
        let mut book = TwoStageBook::<TestSetup>::new(8);
        book.push(10, TestSetup { trigger_level: 100.0 });
        book.push(12, TestSetup { trigger_level: 101.0 });
        // Predicate only matches the older setup.
        let (idx, _) = book.consume_latest(14, |_, s| s.trigger_level == 100.0).unwrap();
        assert_eq!(idx, 10);
    }

    #[test]
    fn oldest_live_advances_monotonically() {
        let mut book = TwoStageBook::<TestSetup>::new(2);
        for i in 0..20 {
            book.push(i, TestSetup { trigger_level: i as f64 });
            let _ = book.consume_latest(i + 1, |_, _| false);
        }
        // After 20 iterations the pruning pointer should have advanced.
        assert!(book.oldest_live > 0);
    }

    #[test]
    fn matches_reference_inline_scan() {
        // Build a random sequence of push/consume events and compare
        // TwoStageBook to the reference Vec+consumed pattern used in
        // opus47_2 v25.
        let mut book = TwoStageBook::<TestSetup>::new(6);
        let mut ref_slots: Vec<(usize, TestSetup, bool)> = Vec::new();
        let ttl = 6usize;

        // Deterministic schedule: push a setup every 3 bars, attempt
        // trigger on every bar with a predicate that depends on bar idx.
        for i in 0..200 {
            if i % 3 == 0 {
                let s = TestSetup { trigger_level: i as f64 };
                book.push(i, s);
                ref_slots.push((i, s, false));
            }
            let pred = |_: usize, s: &TestSetup| s.trigger_level as usize % 5 == i % 5;

            let ref_match: Option<usize> = ref_slots
                .iter()
                .enumerate()
                .rev()
                .find(|(_, (bar, setup, consumed))| {
                    !consumed
                        && *bar < i
                        && i - *bar <= ttl
                        && pred(*bar, setup)
                })
                .map(|(slot_idx, _)| slot_idx);

            let book_match = book.consume_latest(i, pred);

            match (ref_match, book_match) {
                (None, None) => {}
                (Some(slot_idx), Some((bar, _))) => {
                    assert_eq!(
                        ref_slots[slot_idx].0, bar,
                        "disagreement at bar {i}"
                    );
                    ref_slots[slot_idx].2 = true;
                }
                (a, b) => panic!("mismatch at bar {i}: ref={a:?} got={b:?}"),
            }
        }
    }
}
