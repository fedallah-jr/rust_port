# Rust Research Program

This file defines the research rules and the codebase API for Rust
strategy development under `rust_port/`.

**Read this file in full (Part 1 and Part 2) before taking any research
action.** Part 2 is load-bearing — Part 1 rules refer to it, and strategies
that skip it tend to re-implement (badly) indicators, gates, and key-level
helpers that are already shipped.

## Contents

**[Part 1 — Research Rules](#part-1--research-rules)**

- [Objective](#objective)
- [Scoring and Acceptance](#scoring-and-acceptance)
- [Development / Evaluation Separation](#development--evaluation-separation)
- [Session Start](#session-start)
- [Experiment Layout](#experiment-layout)
- [The Experiment Loop](#the-experiment-loop)
- [Outputs and Logging](#outputs-and-logging)
- [Research Lessons](#research-lessons)

**[Part 2 — Codebase API Reference](#part-2--codebase-api-reference)**

- [Core Interface: `ResearchStrategy`](#core-interface-researchstrategy)
- [Candle Struct](#candle-struct)
- [`generate_signals()` Inputs](#generate_signals-inputs)
- [Signal Contract](#signal-contract)
- [Cooldown](#cooldown)
- [Resolution modes](#resolution-modes)
- [Indicators](#indicators)
- [Strategy Primitives (`claude_trader_strategy_blocks`)](#strategy-primitives-claude_trader_strategy_blocks)
- [Point-in-Time Context (`ContextMap`)](#point-in-time-context-contextmap)
- [BTC Structure](#btc-structure)
- [Multi-Timeframe](#multi-timeframe)
- [Calibration](#calibration)

---

# Part 1 — Research Rules

## Objective

Build strategies that **generalize and trade well together**. Both sides
matter: a strategy that scores high only by dominating one regime is not
the goal, and neither is a strategy that is "safe across regimes" but
doesn't actually trade well.

- **Trading performance** is measured by `preference_score`.
- **Generalization** is measured by `generalization_score` — a time-
  stability metric (CV of 2-week PnL buckets).

Both numbers are printed on the `Summary:` line of every run and saved to
`meta.json`, `category_summary.csv`, and `results.tsv`. All tuning happens
on `DEVELOPMENT_WINDOWS`; `EVALUATION_WINDOWS` are a one-time check for a
frozen candidate, not part of the tuning loop.

## Scoring and Acceptance

### Preference Score

`preference_score = coverage_penalty * omega_component * drawdown_component`

- `omega_component = gross_positive_weeks / max(gross_negative_weeks, 1.0)`
- `drawdown_component = (total_pnl / total_weeks) / max(max_drawdown, 5.0%)`
  — the **mean weekly PnL** over the drawdown, not the raw sum. Length-
  invariant so dev (58 weeks) and eval (41 weeks) are directly comparable.
- `coverage_penalty = sqrt(active_weeks / total_weeks) * min(1.0, resolved_trades / 80)`

Eligibility gate: `total_pnl > 0`. Practical target: `100+` resolved trades —
low-trade, low-coverage strategies get penalised hard even if raw PnL looks
acceptable. Tie-breakers after `preference_score`: `generalization_score`,
`sortino_ratio`, `weekly_win_rate`, `profit_factor`, `total_pnl`.

### Generalization Score

A pure time-consistency measure, agnostic to absolute performance. A
strategy that earns a steady +0.1% per 2-week bucket scores near the
maximum — consistency is the question, magnitude belongs to `preference_score`.

```
1. Sort all windows chronologically.
2. Group consecutive windows into disjoint 2-window (14-day) buckets.
   bucket_pnl = sum of total_pnl_pct across the 2 windows.
3. mean = mean(bucket_pnls)
   std  = population std(bucket_pnls)
   cv   = std / max(|mean|, 0.1)         # 0.1 pp floor prevents blow-up
4. generalization_score = 1 / (1 + cv)
```

Range `(0, 1]`. `→1.0` = flat PnL across every bucket; `→0.0` = lumpy. The
`|mean|` floor of 0.1 pp keeps CV finite when equity barely moves. Odd
window counts leave the last bucket with a single window.

`meta.json → generalization` carries: `score`, `cv`, `bucket_count`,
`mean_bucket_pnl`, `std_bucket_pnl`, `bucket_windows`, `mean_floor_pp`.
Read `generalization.cv` directly — a CV well above 1 means lumpy PnL,
usually one or two good bursts carrying the rest.

### Acceptance criteria

A "promising candidate" requires **both** of the following on dev:

1. `preference_score` is strong in absolute terms, AND
2. `generalization_score ≥ ~0.3`.

A strong `preference_score` paired with `generalization_score < 0.2` means
lumpy PnL carried by a handful of lucky buckets — a specialist, not a
generalist, and a near-certain eval failure. **Do not freeze such a
candidate for `--windows eval`.**

## Development / Evaluation Separation

This is the most important rule.

All tuning happens on `DEVELOPMENT_WINDOWS`. `EVALUATION_WINDOWS` must not
be used to tune parameters, choose between near-identical variants,
justify rule changes from holdout-specific behaviour, or run an implicit
parameter search.

After running `--windows eval`, you may only read **aggregate metrics**:
`preference_score`, `generalization_score`, `total_pnl`, `max_drawdown`,
`profit_factor`, `sortino_ratio`, `weekly_win_rate`, and the dev-vs-eval
score gap. Use them solely for the accept/reject decision.

You must NOT read, inspect, or let any of the following influence strategy
changes, even indirectly, when they come from an eval run:

- `trades.csv` (individual trades)
- per-symbol / per-pattern / per-regime breakdowns (`per_symbol.csv`,
  `per_pattern.csv`, `per_regime_vs_pattern.csv`)
- `calibration_log.txt`
- per-bucket or per-category `generalization` breakdowns

If a candidate fails to generalise, diagnose using **development outputs
only** (where everything above is freely inspectable), then return with a
materially new idea.

Short version: dev outputs are fully open; eval outputs are restricted to
aggregate scores for accept/reject.

## Session Start

Before the user explicitly approves a research direction, you may only:

- read this file in full
- read other docs and relevant code
- summarise the existing workflow, constraints, and available APIs

You may **not** scaffold crates, create strategy files, run evaluations
or validation, choose a direction, or start the loop. Do not infer
approval from silence, context, or unrelated follow-up questions.

After approval:

1. Scaffold a new experiment under `rust_port/research/<name>/`.
2. Implement the strategy in that experiment crate.
3. Enter the experiment loop.

## Experiment Layout

Each experiment is its own Cargo crate:

```text
rust_port/research/<name>/
  Cargo.toml
  src/lib.rs
  src/main.rs
  results.tsv
```

**Folder name:** `[model_name]_[effort]_[date]-[N]` where

- `model_name` — model performing the research (e.g. `OPUS4.7`, `GPT5.4`)
- `effort` — session effort level (e.g. `max`, `high`, `mid`)
- `date` — `DD.M.YY` (e.g. `11.4.26` for April 11 2026)
- `N` — `max(existing_numbers) + 1` (check `rust_port/research/` first)

Example: `OPUS4.7_max_11.4.26-1`.

Scaffold with:

```bash
cargo run -p ct-scaffold -- my_strategy
```

### Containment (absolute)

All research work lives inside `rust_port/research/<your-experiment>/`.
**Do not edit anything outside that folder** — this includes everything
under `crates/` (runtime, indicators, strategy_blocks, models,
calibration, research_runtime) and every other experiment folder. If a
shared crate appears buggy or missing a feature, work around it inside
your experiment crate; do not patch the shared code.

One experiment = one crate. Strategy-specific helpers live in that crate.
Do not add strategies by editing central registries — the scaffold tool
creates the expected layout.

## The Experiment Loop

The loop is an infinite cycle: **DEVELOP → EVALUATE → IMPROVE → repeat.**

### Starting points for a new experiment

1. OHLCV only at `analysis_interval()`
2. A small indicator set from `claude_trader_indicators`
3. Explicit rules built from `claude_trader_strategy_blocks` detectors and
   gates — prefer these over hand-rolled equivalents

Reach for funding rates, key levels, BTC bias, or additional timeframes
only when the hypothesis clearly calls for them.

### Each cycle

1. **Develop** on `DEVELOPMENT_WINDOWS`:
   ```bash
   cargo run --release -p ct-research-<name> -- eval --windows dev
   # or: ./rust_port/bin/run_research <name> eval --windows dev
   ```
2. **Refine** from dev outputs, code understanding, and external research
   (papers, blog posts, ideas — all welcome).
3. **Freeze** once the candidate hits both acceptance criteria (strong
   `preference_score` and `generalization_score ≥ ~0.3`). Record the
   hypothesis, symbols, and parameters.
4. **Validate** no-lookahead replay on dev:
   ```bash
   cargo run --release -p ct-research-<name> -- validate --windows dev
   ```
   Validation replays each signal with truncated candles, a clipped
   `ContextMap`, a clipped `HtfData`, and the calibration params active at
   that time. On failure it writes `validation_result.json`.
5. **Evaluate** on `EVALUATION_WINDOWS`:
   ```bash
   cargo run --release -p ct-research-<name> -- eval --windows eval
   ```
6. **Log** the row in `results.tsv` and continue with the next iteration.

### Loop rules

- After seeing eval results, do not make small parameter changes justified
  by specific eval outcomes.
- If a candidate fails to generalise, diagnose from dev outputs and return
  with a **materially new idea** — not a near-identical holdout variant.
- Preserve failed-candidate reasoning in `results.tsv` so the same bad
  idea is not retried.
- Use `search_parameters()` from `claude_trader_calibration` for any grid
  search — never roll your own nested loops (see [Calibration](#calibration)).
- Do not hardcode dates, symbols, or outcomes from known windows.
- Do not emit signals with `signal_date` outside `[start, end)`.

### NEVER STOP

Once the loop has begun, **do not** pause to ask the human whether to
continue. No "is this a good stopping point?", no "should I keep going?".
The loop runs indefinitely until the human interrupts.

## Outputs and Logging

Each `eval` run saves timestamped outputs under:

```text
outputs/strategy_eval/<slug>_<window>_<mode>_<timestamp>/
  trades.csv
  meta.json
  category_summary.csv
  per_pattern.csv
  per_symbol.csv
  per_regime_vs_pattern.csv
  calibration_log.txt   (only when calibration is enabled)
```

Each `eval` run **upserts** a single row in the experiment's `results.tsv`
keyed by `strategy_name`:

```text
strategy_name  strategy_description  strategy_score_dev  strategy_score_eval  performance_description
```

One row per `strategy_name` — never multiple rows for the same strategy,
even across repeated runs or dev + eval runs. The runtime finds the
existing row by column 0 and updates it in place; a new row is appended
only when no match exists.

Column ownership and per-run behaviour:

| Column | Owner | Behaviour on re-run |
|---|---|---|
| `strategy_name` | strategy code (`name()`) | Used as the upsert key — do not change within a version. |
| `strategy_description` | strategy code (`description()`) | **Refreshed every run** from `description()`. Edit the method in code; manual edits to this column are overwritten. |
| `strategy_score_dev` | runtime | Filled by `--windows dev` runs; left as `-` until a dev run exists; unchanged by eval runs. |
| `strategy_score_eval` | runtime | Filled by `--windows eval` runs; left as `-` until an eval run exists; unchanged by dev runs. |
| `performance_description` | the researcher | Written once on first insert (auto-generated metrics as a starting point), then **preserved on every subsequent run**. Freely editable — record what worked, what failed, and why. This is what prevents retrying the same bad idea. |

Practical consequence: to explain a version, either (a) edit the
`description()` method so the change is reflected in code and re-runs
keep the column in sync, or (b) rewrite `performance_description`
directly in `results.tsv` — your edit will survive future runs.

`validate` does not touch `results.tsv`; on failure it writes
`validation_result.json`.

### Reading dev outputs

Read these every iteration:

- **`Summary:` console line** — `Pref X.XXX | Gen Y.YYY`. The primary
  score signal.
- **`Generalization:` console line** — score, CV, mean/bucket, std/bucket,
  bucket count.
- **`category_summary.csv`** and **`per_regime_vs_pattern.csv`** — show
  which regime is weak when `generalization_score` is low.
- **`per_pattern.csv`** — one row per `signal.pattern`. A pattern with
  many trades but `profit_factor ≈ 1` is adding noise without edge (a
  sub-strategy ablation signal).
- **`per_symbol.csv`** — edge distribution across the universe.
  **Diagnostic only.** Do not cherry-pick symbols on dev — dropping dev-
  loser symbols is a classic overfit trap.
- **`calibration_log.txt`** — each calibration interval's time range and
  selected params. Use it to check whether params are stable or whipsawing.
  Not printed to the console; read it from the output directory.
- **`meta.json → overall`** — scalar rollups (PnL, PF, Sortino, MDD, Omega).
- **`meta.json → generalization`** — the `generalization.*` fields listed
  in [Generalization Score](#generalization-score).

All of the above are freely inspectable on dev and restricted to aggregate
scores only on eval (see
[Development / Evaluation Separation](#development--evaluation-separation)).

## Research Lessons

Each experiment crate should contain a `LESSONS.md` capturing empirical
findings so future researchers do not re-derive the same dead ends.

**When to write:** only when the human explicitly interrupts and asks for
it. Do not write `LESSONS.md` proactively during the loop.

**What to write:** the most important empirical findings — things NOT
obvious from reading the API docs or code. Structure it as:

```markdown
# Research Lessons — <experiment_name>

## What Doesn't Work (and Why)
<!-- Approaches that looked promising but failed. Include version number,
     what you tried, and WHY it failed. -->

## What Actually Moves the Needle
<!-- Changes that produced real, generalisable improvements.
     Distinguish dev-only wins from dev+eval wins. -->

## Overfitting Traps
<!-- Patterns that improved dev but failed eval. Include version numbers. -->

## Structural Constraints
<!-- Fundamental limits of the market, timeframe, or instrument.
     Things to accept rather than fight. -->

## Open Questions
<!-- Promising directions not yet explored. -->
```

**Where:** `rust_port/research/<experiment_name>/LESSONS.md`, next to
`results.tsv`. Read existing `LESSONS.md` files before starting new
experiments to avoid repeating known-bad ideas.

---

# Part 2 — Codebase API Reference

## Core Interface: `ResearchStrategy`

Every experiment implements `claude_trader_research_runtime::ResearchStrategy`.

Required methods:

| Method | Purpose |
|---|---|
| `name()` | Display name used in outputs and `results.tsv` |
| `symbols()` | Symbols traded by the strategy |
| `indicator_columns()` | Indicator names to precompute warmup for |
| `cooldown_spec(signal)` | Return a `CooldownSpec { key, hours }` for each emitted signal. The runtime enforces cooldown **globally across a run** using the returned key — strategies MUST NOT track cooldown internally. See [Cooldown](#cooldown). |
| `generate_signals(...)` | Main backtest signal-generation method |

Optional methods:

| Method | Purpose |
|---|---|
| `description()` | Human-readable hypothesis / what changed vs. the previous version. Written to the `strategy_description` column of `results.tsv` every run. Default: `name().to_string()`. |
| `extra_warmup_bars()` | Extra bars beyond indicator-derived warmup; default `100` |
| `analysis_interval()` | Candle timeframe (e.g. `"1h"`, `"4h"`, `"15m"`); default `"1h"`. Runtime fetches candles at this interval. |
| `additional_intervals()` | Extra candle timeframes delivered via `htf.additional_candles`. Default empty. See [Multi-Timeframe](#multi-timeframe). |
| `extra_warmup_bars_per_interval()` | Per-interval warmup bar counts for additional intervals. Default empty. |
| `market_data_request()` | Override the base `MarketDataRequest` (e.g. to add agg-trades). Default OHLCV at `analysis_interval()`. Funding rates, key levels, and BTC structure are auto-derived from `required_context()` — do not set them here. |
| `required_context()` | Declare point-in-time context dependencies (`ContextKey::BtcStructure`, `ContextKey::KeyLevels(sym)`, `ContextKey::Funding(sym)`). Drives fetching, warmup, and `ContextMap` population. |
| `calibration_config()` | Enable rolling calibration |
| `calibrate()` | Run calibration on lookback candles and return active params |

The runtime entry point is in `src/main.rs`. The strategy logic belongs in
`src/lib.rs`.

## Candle Struct

Each candle passed to the strategy has these fields:

| Field | Type | Notes |
|---|---|---|
| `open_time` | `DateTime<Utc>` | Candle open timestamp |
| `close_time` | `DateTime<Utc>` | Candle close timestamp — use this for signal_date and time comparisons |
| `open` | `f64` | Opening price |
| `high` | `f64` | High price |
| `low` | `f64` | Low price |
| `close` | `f64` | Closing price |
| `volume` | `f64` | Total volume |
| `taker_buy_volume` | `f64` | Taker buy volume (used by `volume_delta` indicator) |

## `generate_signals()` Inputs

The strategy works directly with raw inputs:

| Argument | Type | Meaning |
|---|---|---|
| `candles` | `&BTreeMap<String, &[Candle]>` | Per-symbol candles at `analysis_interval()`, including warmup bars before `start`. Borrowed slices, not owned `Vec`s; `BTreeMap` gives deterministic symbol iteration order. |
| `start`, `end` | `DateTime<Utc>` | Only emit signals with `signal_date` in `[start, end)` |
| `active_params` | `&HashMap<String, serde_json::Value>` | Rolling calibration params active for this slice |
| `ctx` | `&ContextMap` | Unified point-in-time accessor. Call `ctx.context_at(&key, t)` for BTC bias, key levels, or funding context declared via `required_context()`. Already clipped to `end` by the runtime. |
| `htf` | `&HtfData` | Higher-timeframe candles and their precomputed indicators for every interval declared in `additional_intervals()`. Already clipped to `end` by the runtime. |

Important:

- `candles` are the primary data source
- compute indicators inside the strategy from those candles
- point-in-time context (BTC bias, key levels, funding) is reachable only
  via `ctx.context_at(&key, t)` — there is no other handle on those streams
- the runtime handles warmup, calibration orchestration, context assembly,
  validation replay state, trade resolution, and output saving

## Signal Contract

The strategy returns `Vec<Signal>`.

| Field | Type | Default | Notes |
|---|---|---|---|
| `signal_date` | `DateTime<Utc>` | required | Signal timestamp (must be in `[start, end)`) |
| `position_type` | `PositionType` | required | `Long` or `Short` |
| `ticker` | `String` | required | Symbol like `BTCUSDT` |
| `pattern` | `String` | `""` | Strategy pattern name (e.g. `"buy_dip"`, `"sell_rip"`). Written to `trades.csv` for per-pattern analysis. |
| `tp_pct` / `tp_price` | `Option<f64>` | `None` | At least one TP field required. `tp_pct` is a percentage: `3.0` = 3% above entry (fees are added on top). |
| `sl_pct` / `sl_price` | `Option<f64>` | `None` | At least one SL field required. `sl_pct` is a percentage: `1.5` = 1.5% below entry (fees are subtracted from the cushion). |
| `leverage` | `f64` | `1.0` | Multiplier on PnL — a 2% move at 5x leverage = 10% return. |
| `market_type` | `MarketType` | `Futures` | |
| `taker_fee_rate` | `f64` | `0.0005` | 0.05% per side. The resolver applies round-trip fees (2x) to TP/SL price calculation. |
| `entry_price` | `Option<f64>` | `None` | `None` = market order. In exact mode fills via agg trades after `entry_delay_seconds`; in approximate mode fills at the open of the next 1m candle after `signal_date` (see Resolution modes). `Some(price)` = limit order (fills if price is reached within `fill_timeout_seconds`; limit orders are only honored in exact mode). |
| `fill_timeout_seconds` | `i64` | `3600` | Limit order fill window in seconds. Only used when `entry_price` is `Some` and the resolver is in exact mode. |
| `entry_delay_seconds` | `Option<i64>` | `None` | Seconds to wait after `signal_date` before market-order entry. Default 3s when `None`. Ignored in approximate mode. |
| `max_holding_hours` | `i64` | `72` | Must be positive. Trade is force-closed after this duration. |
| `size_multiplier` | `f64` | `1.0` | Position size scaling factor. |
| `metadata` | `HashMap<String, Value>` | `{}` | Arbitrary strategy metadata. |

Signal invariants:

- at least one of `tp_pct` or `tp_price` must be set
- at least one of `sl_pct` or `sl_price` must be set
- `max_holding_hours` must be positive
- `signal_date` must be inside `[start, end)`

## Cooldown

Cooldown is the **runtime's** responsibility. Strategies emit every candidate
signal and declare a `CooldownSpec` per signal via `cooldown_spec(&self, signal: &Signal)`.
The runtime then filters the full signal stream **globally across the run**
(not per window, not per calibration period) before backtesting.

```rust
use claude_trader_models::{CooldownSpec, CooldownKey, Signal};

// Default: per (symbol, direction). Same symbol, opposite side = different key.
fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
    CooldownSpec::symbol_side(signal, 12.0)
}
```

Built-in key constructors:

| Constructor | Groups by |
|---|---|
| `CooldownKey::symbol(ticker)` | symbol (long and short share the same bucket) |
| `CooldownKey::symbol_side(ticker, side)` | symbol + direction |
| `CooldownKey::symbol_pattern(ticker, pattern)` | symbol + `signal.pattern` |
| `CooldownKey::pattern(pattern)` | pattern globally |
| `CooldownKey::custom(s)` | arbitrary grouping — `s` is namespaced under `custom:` so it cannot collide with the built-ins |

Build a `CooldownSpec` directly when you need a non-default grouping:

```rust
fn cooldown_spec(&self, signal: &Signal) -> CooldownSpec {
    // Example: per symbol+pattern, 24h for "breakout", 8h otherwise.
    let hours = if signal.pattern == "breakout" { 24.0 } else { 8.0 };
    CooldownSpec {
        key: CooldownKey::symbol_pattern(&signal.ticker, &signal.pattern),
        hours,
    }
}
```

Invariants:

- `hours` must be finite and `>= 0.0`. `hours == 0.0` means no cooldown for that
  signal.
- Two signals that share a key must carry the same `hours` in the same run.
  Encode different regimes as different keys. The runtime panics on inconsistent
  hours under debug builds and normalizes to integer seconds in release builds.
- The filter runs once per evaluation, over all raw candidate signals sorted by
  `signal_date`. Calibration boundaries do not reset cooldown state.

## Resolution modes

The resolver runs in one of two modes, selected by the `approximate` flag on
`backtest_signal`:

- **Exact**: market orders fill from agg trades after `entry_delay_seconds`
  (with a minute-candle open fallback when no agg trades exist in the fill
  window); limit orders fill via agg trades within `fill_timeout_seconds`.
- **Approximate**: market and limit orders are collapsed to a single uniform
  rule — for a signal whose `signal_date` lies in minute `m`, the entry is the
  **open of the next 1m candle** (`open_time = m + 1min`), stamped with that
  candle's `open_time`. 1-minute candles are used regardless of the strategy's
  native timeframe. `entry_delay_seconds`, `entry_price`, and
  `fill_timeout_seconds` are ignored — the minute-granularity approximation
  already subsumes sub-minute delays and limit-fill timing. If the
  next-minute candle is absent, the trade is marked `Unfilled` rather than
  silently approximating further.

## Indicators

Rust indicators are provided by `claude_trader_indicators`, which exposes:

- `compute_indicators(&OhlcvFrame, &[&str])`
- `required_warmup(&[&str])`

Common public indicator names:

- `rsi_14`
- `atr_14`
- `atr_72_avg`
- `atr_ratio`
- `ret_24h`
- `ret_48h`
- `ret_72h`
- `vol_sma_20`
- `vol_ratio`
- `ema_20`
- `bb_upper`
- `bb_lower`
- `bb_pct_b`
- `bb_width`
- `kc_upper`
- `kc_lower`
- `squeeze_on`
- `squeeze_count`
- `mom_slope`
- `body`
- `body_ratio`
- `adx_14`
- `volume_delta`
- `cvd`
- `t3`
- `vwap_20` / `vwap_48`
- `vwap_dev_20` / `vwap_dev_48`
- `poc_48`
- `poc_dev_48`

The scaffold-generated `src/lib.rs` (see `cargo run -p ct-scaffold -- <name>`)
shows the standard pattern:

1. build an `OhlcvFrame` from a symbol's candle slice
2. call `compute_indicators(...)`
3. iterate over the candle indices that fall inside `[start, end)`

### VWAP and POC

`vwap_20` / `vwap_48` are rolling typical-price (HLC/3) VWAPs over the
last 20 and 48 bars respectively. `vwap_dev_N` is percent deviation of
current close from `vwap_N`, positive when price is above the fair-value
mean — useful as an overextension filter.

`poc_48` is the rolling Point of Control over the last 48 bars: the
centre price of the bin with the highest traded volume. Each bar
attributes its entire volume to the bin containing its close, and the
price range is split into 50 equal bins. `poc_dev_48` is percent
deviation of current close from `poc_48` — price significantly above or
below the volume-concentration area often precedes mean reversion.

Session-anchored VWAPs (reset at UTC midnight, Monday open, etc.) are
not part of the indicator framework because `OhlcvFrame` does not carry
timestamps. Strategies that need session VWAP should compute it inline
from the `Candle` slice, which has `open_time` and `close_time`.

## Strategy Primitives (`claude_trader_strategy_blocks`)

`crates/strategy_blocks` is a shared crate of pure, composable detectors
and gates. It sits below `claude_trader_research_runtime` and is designed
to be imported directly by research crates — it exists so that every
experiment doesn't re-implement bearish/bullish divergence, Donchian
breakouts, key-level pierce logic, or BTC-bias / funding / 4h HTF gates.

### Available blocks

| Module | Exports | Purpose |
|---|---|---|
| `events` | `DivergenceDetection`, `BreakoutDetection`, `LevelPierce`, `Direction` | Shared `Copy` event structs returned by detectors. |
| `divergence` | `bearish_double_divergence`, `bullish_double_divergence`, `DivergenceParams` | CVD + RSI double-divergence scan over a lookback window. |
| `breakout` | `donchian_high_break`, `donchian_low_break` | Rolling N-bar Donchian break detection with a minimum-move gate. |
| `levels` | `pierced_resistance`, `pierced_support`, `LevelPierceParams` | Key-level pierce/rejection detectors ranked by tier (monthly > weekly > daily). |
| `setup` | `TwoStageBook<S>` | Generic setup → trigger state machine with per-bar TTL and consumption. |
| `gates` | `btc_bias_bullish`, `btc_bias_bearish`, `funding_crowded_long`, `funding_crowded_short`, `htf_4h_not_strongly_bullish`, `htf_4h_not_strongly_bearish`, `FundingGateParams` | Stateless filter helpers over `MarketBias`, `FundingRate`, and 4h candles. |

Everything in this crate is inline-able and allocation-free on the hot
path. `TwoStageBook<S>` allocates only when its internal `Vec` grows;
use `TwoStageBook::with_capacity` to pre-size when the number of
setups-per-call is bounded.

### Purity contract (IMPORTANT)

**Every function in `claude_trader_strategy_blocks` is a pure function of
its arguments.** This is what makes the no-lookahead validator work: the
runtime replays `generate_signals()` with truncated inputs (candles,
`ContextMap`, `HtfData`, calibration params) and requires identical
output. Your strategy code must preserve the same purity — any of the
following silently breaks the guarantee:

- caching between calls (e.g. a `static` indicator cache)
- `thread_local!` / `static mut` / `Mutex`-guarded state
- I/O of any kind (file, network, clock)
- reading data outside the slices/references passed in

The only mutable state allowed is per-invocation: e.g. a
`TwoStageBook<S>` constructed on the stack by `generate_signals()` and
dropped when the function returns.

### Example usage

The pure detectors (`bearish_double_divergence`, `donchian_*`,
`pierced_*`) take plain slices and are unchanged. The gate helpers
(`btc_bias_*`, `funding_*`, `htf_4h_*`) still take their original raw
shapes (`&HashMap<DateTime<Utc>, MarketBias>`, `&[FundingRate]`,
`&[Candle]`), but strategies now receive context through `&ContextMap` —
so either read from `ctx.context_at(...)` directly, or wrap the gate with
a small adapter:

```rust
use claude_trader_models::{ContextKey, ContextMap, ContextValue, MarketBias};
use claude_trader_strategy_blocks::{
    bearish_double_divergence, DivergenceParams, TwoStageBook,
};

fn bias_bullish(ctx: &ContextMap, t: DateTime<Utc>) -> bool {
    matches!(
        ctx.context_at(&ContextKey::BtcStructure, t),
        Some(ContextValue::Bias(MarketBias::Bullish)),
    )
}

fn funding_rate_at(ctx: &ContextMap, sym: &str, t: DateTime<Utc>) -> Option<f64> {
    match ctx.context_at(&ContextKey::Funding(sym.to_string()), t) {
        Some(ContextValue::Funding(f)) => Some(f.rate),
        _ => None,
    }
}

let div_params = DivergenceParams {
    lookback: 24, exclude_recent: 4,
    min_price_diverg_pct: 0.003, min_rsi_gap: 0.0,
};

let mut short_book = TwoStageBook::<MySetup>::with_capacity(6, 32);
for i in warmup..symbol_candles.len() {
    if let Some(det) = bearish_double_divergence(&highs, &cvd, &rsi, i, &div_params) {
        short_book.push(i, MySetup { pivot_low: lows[i], detection: det });
    }
    let close = symbol_candles[i].close;
    let close_time = symbol_candles[i].close_time;
    if let Some((setup_idx, setup)) = short_book.consume_latest(i, |_, s| close < s.pivot_low) {
        let funding_high = funding_rate_at(ctx, symbol, close_time)
            .map_or(false, |r| r > 0.0003);
        if funding_high && !bias_bullish(ctx, close_time) {
            emit_short_signal_at(i, setup_idx, setup);
        }
    }
}
```

## Point-in-Time Context (`ContextMap`)

All non-OHLCV scalar signals — BTC structural bias, key levels, and funding
context — reach the strategy through a single unified accessor:
`ctx.context_at(&key, t)`. Strategies have no other handle on these streams.

### Declaring what you need

Every context dependency is declared via `required_context()`. The runtime
uses this list to drive fetching (per symbol), warmup (30 days for funding,
inlined for the others), and `ContextMap` population:

```rust
fn required_context(&self) -> Vec<ContextKey> {
    let mut v = vec![ContextKey::BtcStructure];
    for sym in self.symbols() {
        v.push(ContextKey::Funding(sym));       // per-symbol funding context
        v.push(ContextKey::KeyLevels(sym));     // per-symbol key levels
    }
    v
}
```

Asking for `Funding("BTCUSDT")` while trading ETH is fine — cross-symbol
context works, and the runtime fetches exactly the symbols you request, not
the full `symbols()` list.

Funding and key levels are driven exclusively by `required_context()` —
there are no flags for them in `MarketDataRequest`. The only
`DataRequirement` variants today are `Ohlcv` and `AggTrades`, so
`market_data_request()` exists solely for OHLCV- and agg-trades-related
knobs.

### Reading context

| Key | Returned variant | Notes |
|---|---|---|
| `ContextKey::BtcStructure` | `ContextValue::Bias(MarketBias)` | BTC daily structural bias. Source timestamps are daily candle close_times, independent of analysis interval. |
| `ContextKey::KeyLevels(sym)` | `ContextValue::KeyLevels(KeyLevels)` | Per-symbol structural levels. |
| `ContextKey::Funding(sym)` | `ContextValue::Funding(FundingContext)` | Computed on demand at query time `t` — 7d/30d rolling windows are evaluated against `t`, not the most recent funding event. |

```rust
use claude_trader_models::{ContextKey, ContextMap, ContextValue, MarketBias};

// BTC bias at the current bar's close_time
let is_bearish = matches!(
    ctx.context_at(&ContextKey::BtcStructure, close_time),
    Some(ContextValue::Bias(MarketBias::Bearish)),
);

// Key levels
if let Some(ContextValue::KeyLevels(levels)) =
    ctx.context_at(&ContextKey::KeyLevels(symbol.clone()), close_time)
{
    let pdh = levels.pdh;
    // ...
}

// Funding context
if let Some(ContextValue::Funding(f)) =
    ctx.context_at(&ContextKey::Funding(symbol.clone()), close_time)
{
    f.zscore_30d;    // current rate as z-score vs 30-day distribution
    f.cumulative_7d; // sum over last 7 days (~21 periods)
    f.rate_change;   // diff from previous funding period
    f.rate;          // raw current rate
}
```

`context_at` returns `None` before the first source event, and for funding
specifically when fewer than 3 rates precede `t`.

### `KeyLevels` fields

Key levels include:

- PDH / PDL and daily equilibrium
- weekly, monthly, quarterly, yearly structural levels
- Monday range
- Asia / London / New York session levels

### `FundingContext` vs raw `FundingRate`

Strategies see `FundingContext` (computed on demand from private raw rates).
The raw `FundingRate` slice is no longer exposed. If a strategy needs the
raw value rather than the computed context, read `FundingContext.rate`.

### Invariant

`ContextMap` exposes no iterator and no series accessor. For any bar at
close_time `t`, the only values a strategy can read are those sourced from
events with `ts <= t`. The runtime clips the `ContextMap` per period (and
the validator per signal) before handing it to the strategy.

## BTC Structure

Include `ContextKey::BtcStructure` in `required_context()` to enable BTC
daily structural bias. Read it at any bar's `close_time`:

```rust
use claude_trader_models::{ContextKey, ContextValue, MarketBias};

match ctx.context_at(&ContextKey::BtcStructure, close_time) {
    Some(ContextValue::Bias(MarketBias::Bearish)) => { /* bear regime */ }
    Some(ContextValue::Bias(MarketBias::Bullish)) => { /* bull regime */ }
    _ => { /* neutral or no bias yet */ }
}
```

The series is timestamped at the daily BTC candle close_times (source
events). `context_at` does a `partition_point` lookup, so the returned bias
is the most recent daily event with `ts <= close_time` — analysis-interval
independent.

The `btc_bias_bullish` / `btc_bias_bearish` gates in
`claude_trader_strategy_blocks::gates` take a
`&HashMap<DateTime<Utc>, MarketBias>`. When you want them driven off the
`ContextMap`, wrap the read yourself:

```rust
fn bias_bullish(ctx: &ContextMap, t: DateTime<Utc>) -> bool {
    matches!(
        ctx.context_at(&ContextKey::BtcStructure, t),
        Some(ContextValue::Bias(MarketBias::Bullish)),
    )
}
```

## Multi-Timeframe

Strategies can request candles at additional timeframes beyond
`analysis_interval()`. The primary candles arrive in the `candles` parameter
of `generate_signals()` as before; additional timeframes are delivered
through the `htf: &HtfData` parameter.

### Enabling multi-timeframe

Override `additional_intervals()`:

```rust
fn additional_intervals(&self) -> Vec<&str> {
    vec!["1h"]  // e.g. primary is "4h", also want 1h for entry timing
}
```

Optionally declare per-interval warmup bars:

```rust
fn extra_warmup_bars_per_interval(&self) -> HashMap<&str, usize> {
    let mut m = HashMap::new();
    m.insert("1h", 200);  // 200 bars of 1h warmup
    m
}
```

Optionally declare per-interval indicator columns so the runtime
precomputes them into `htf.additional_indicators`:

```rust
fn indicator_columns_per_interval(&self) -> HashMap<&str, Vec<&str>> {
    let mut m = HashMap::new();
    m.insert("1h", vec!["ema_20", "atr_14"]);
    m
}
```

### Reading HTF candles and indicators

In `generate_signals()`, access them from `htf`:

```rust
fn generate_signals(&self, candles: &BTreeMap<String, &[Candle]>,
    start: DateTime<Utc>, end: DateTime<Utc>,
    active_params: &HashMap<String, serde_json::Value>,
    ctx: &ContextMap,
    htf: &HtfData,
) -> Vec<Signal> {
    for (sym, bars_4h) in candles {
        // `bars_4h` is `&&[Candle]` — primary 4h candles for pattern detection.
        // ...

        // Additional 1h candles for entry timing
        if let Some(bars_1h) = htf.additional_candles
            .get("1h")
            .and_then(|m| m.get(sym))
        {
            // bars_1h is &Vec<Candle>, sorted by close_time
        }

        // Precomputed indicators on the 1h candles (aligned 1:1 with bars_1h)
        if let Some(ind_1h) = htf.additional_indicators
            .get("1h")
            .and_then(|m| m.get(sym))
        {
            let ema_20 = ind_1h.get("ema_20");  // Option<&Vec<f64>>
        }
    }
}
```

### Restrictions

- **1m is not supported** as an additional interval (too large for memory).
  Use `analysis_interval("1m")` instead.
- `analysis_interval()` must not appear in `additional_intervals()`.
- Duplicates are rejected.
- Additional candles are **clipped to the signal window end** — no lookahead.
- Additional candles are available in `calibrate()` via the clipped `htf`
  parameter.

### How it flows at runtime

1. Runtime fetches candles at each additional interval with per-interval
   warmup (covering indicator warmup + `extra_warmup_bars_per_interval()` +
   calibration lookback if enabled).
2. Candles and precomputed indicators are assembled into a single `HtfData`
   once per run.
3. Before each `generate_signals()` call, `HtfData` is clipped to the
   signal window end (no future data) via `HtfData::truncated_at`.
4. Validator replays clone + clip the same `HtfData` per signal for
   bit-identical no-lookahead verification.

## Calibration

Rolling calibration lets the strategy re-fit parameters periodically using a
lookback window of historical candles.

### Enabling calibration

Override `calibration_config()`:

```rust
fn calibration_config(&self) -> Option<CalibrationConfig> {
    Some(CalibrationConfig {
        interval_hours: 168,   // recalibrate every 7 days
        lookback_hours: 720,   // use 30 days of history for each calibration
    })
}
```

### Implementing `calibrate()`

The runtime calls `calibrate()` at each calibration boundary with the
lookback candles. Return the best parameters as
`HashMap<String, serde_json::Value>`:

```rust
fn calibrate(
    &self,
    candles: &BTreeMap<String, &[Candle]>,
    ctx: &ContextMap,
    htf: &HtfData,
) -> Option<HashMap<String, serde_json::Value>> {
    // Build indicator frame from candles, define a param_space, score
    // each combination, and return the best params.
    // `ctx` and `htf` are clipped to the lookback window end.
    // See the search_parameters() utility below.
}
```

### Grid search utility

The `claude_trader_calibration` crate provides `search_parameters()` for
exhaustive grid search (max 50,000 combinations). **This function scores
candidates in parallel via rayon. All calibration grid searches must use
`search_parameters()` — do not implement manual nested loops.** The
`score_fn` closure runs concurrently and must be `Sync`.

Capture whatever per-search context the strategy needs (candles, HTF data,
a precomputed indicator table, etc.) in the closure:

```rust
use claude_trader_calibration::{search_parameters, CalibrationResult};

let mut param_space = HashMap::new();
param_space.insert("rsi_low".into(), vec![json!(25.0), json!(30.0), json!(35.0)]);
param_space.insert("rsi_high".into(), vec![json!(65.0), json!(70.0), json!(75.0)]);

let score_ctx = build_score_context(candles, htf, ctx);  // your own type

let result: Option<CalibrationResult> = search_parameters(
    &param_space,
    |params| {
        // Return Some(score) where higher = better, or None to skip.
        let rsi_low = params["rsi_low"].as_f64().unwrap();
        score_params(params, &score_ctx)
    },
);

if let Some(cal) = result {
    Some(cal.best_params)  // HashMap<String, serde_json::Value>
}
```

Custom scoring uses closure-capture: build whatever per-search context your
strategy needs (a candle slice, a precomputed indicator table, a regime cache —
any types, any shape), capture it by reference in the scoring closure, and
`search_parameters` runs the closure in parallel across every parameter
combination. The closure's captured state must be `Sync`, which `Vec`, `HashMap`,
and plain structs already are.

### How it flows at runtime

1. Runtime divides the evaluation period into intervals of `interval_hours`
2. At each boundary, it extracts `lookback_hours` of candles and calls
   `calibrate()`
3. The returned params are passed as `active_params` to `generate_signals()`
   for that interval
4. Intervals and params are recorded so validation replay uses the correct
   params for each signal's time

The runtime records calibration intervals and reuses the correct params during
no-lookahead validation replay.

### Sub-strategy selection via calibration

Calibration can select between fundamentally different entry patterns, not just
tune numeric thresholds. The mechanism uses a reserved `_sub_strategy` key in
the params map.

**How it works:**

1. Define multiple entry pattern functions (e.g., squeeze release, CVD
   divergence, impulse breakout), each as a standalone function.
2. Create a dispatch function that routes to the correct pattern based on
   `params["_sub_strategy"]`.
3. In `calibrate()`, define a separate param space per sub-strategy. Each
   space includes `"_sub_strategy"` as a single-value axis (e.g.,
   `vec![json!("squeeze")]`) plus the numeric params for that sub-strategy.
4. Run `search_parameters()` independently for each sub-strategy's space.
5. Compare best scores across sub-strategies (first in declared order wins
   ties). Return the winning sub-strategy's `best_params`, which already
   contains `_sub_strategy`.
6. In `generate_signals()`, read `params["_sub_strategy"]` and dispatch to
   the selected pattern only.

**Key conventions:**

- `_sub_strategy` absent from params = run all patterns in priority order
  (backward compatible, no sub-strategy selection)
- `_sub_strategy` set to a valid name = run only that pattern
- `_sub_strategy` set to an unknown name = produce no signals (fail closed)
- Calibration scoring must validate `_sub_strategy` strictly: reject unknown
  names by returning `None` from the score function
- Signal generation may use defaulting for numeric params but must preserve
  the raw `_sub_strategy` value for dispatch

**Parsing contract:**

Use two parsing modes for the params map:

- **Strict** (for `calibrate` / `score_params`): require all numeric keys,
  reject unknown `_sub_strategy` values. Missing or invalid params cause the
  candidate to be skipped (return `None`).
- **Defaulting** (for `generate_signals`): default missing numeric keys to
  safe values, but preserve the raw `_sub_strategy` string so the dispatch
  function can reject unknown values.

**Param space structure:**

Each sub-strategy gets its own param space. Derive all spaces from a shared
`base_param_space()` to avoid drift:

```rust
fn base_param_space() -> HashMap<String, Vec<Value>> {
    let mut s = HashMap::new();
    s.insert("threshold_a".into(), vec![json!(1.0), json!(2.0)]);
    s.insert("threshold_b".into(), vec![json!(0.5), json!(0.8)]);
    s
}

fn sub_strategy_param_spaces() -> Vec<HashMap<String, Vec<Value>>> {
    ["pattern_a", "pattern_b"].iter().map(|&name| {
        let mut space = base_param_space();
        space.insert("_sub_strategy".into(), vec![json!(name)]);
        space
    }).collect()
}
```

**Calibrate loop:**

```rust
fn calibrate(
    &self,
    candles: &BTreeMap<String, &[Candle]>,
    ctx: &ContextMap,
    htf: &HtfData,
) -> Option<HashMap<String, Value>> {
    let score_ctx = build_score_context(candles)?;
    let score_fn = |params: &HashMap<String, Value>| -> Option<f64> {
        score_params_strict(params, &score_ctx)
    };

    let mut best: Option<(f64, HashMap<String, Value>)> = None;
    for space in sub_strategy_param_spaces() {
        if let Some(result) = search_parameters(&space, &score_fn) {
            // Strict >: first sub-strategy in declared order wins ties
            if best.as_ref().map_or(true, |(s, _)| result.best_score > *s) {
                best = Some((result.best_score, result.best_params));
            }
        }
    }
    best.map(|(_, params)| params)
}
```

**Scope:** v1 supports shared-schema sub-strategies only (all sub-strategies
use the same numeric param grid). Per-sub-strategy schemas (different params
per pattern) would require extending the param struct and spaces — this is a
future extension.
