#!/usr/bin/env python3
"""Golden fixture generator for Rust parity tests.

Imports from the Python codebase, runs each subsystem on canonical inputs,
and serializes outputs as JSON golden references under rust_port/fixtures/.

Usage:
    source ~/.venv/bin/activate
    python rust_port/fixtures/generate.py [--subset SUBSET ...]

Subsets: resolver, indicators, engine, calibration, context, live
Default: all subsets.

Deterministic: same seed, same data, same output every run.
"""

from __future__ import annotations

import argparse
import json
import sys
import os
from datetime import datetime, timedelta, timezone
from pathlib import Path
from random import Random

# Ensure project root is on sys.path
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))

from backtester.models import (
    AggTrade,
    Candle,
    ExitReason,
    PositionType,
    MarketType,
    Signal,
)
from backtester.resolver import compute_pnl, compute_tp_sl_prices
from backtester.engine import _compute_stats, backtest_signal
from backtester.indicators import compute_indicator_frame, required_warmup
from backtester.data import BinanceClient

UTC = timezone.utc
FIXTURES_DIR = Path(__file__).resolve().parent
SEED = 42


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def dt_to_iso(dt: datetime) -> str:
    """Convert datetime to ISO 8601 string with UTC timezone."""
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=UTC)
    return dt.isoformat()


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def write_json(path: Path, data):
    """Write JSON with deterministic formatting."""
    with open(path, "w") as f:
        json.dump(data, f, indent=2, default=_json_default, sort_keys=True)
    print(f"  wrote {path.relative_to(FIXTURES_DIR)}")


def _json_default(obj):
    if isinstance(obj, datetime):
        return dt_to_iso(obj)
    if isinstance(obj, float):
        if obj != obj:  # NaN
            return "NaN"
        if obj == float("inf"):
            return "Infinity"
        if obj == float("-inf"):
            return "-Infinity"
        return obj
    raise TypeError(f"Cannot serialize {type(obj)}")


# ---------------------------------------------------------------------------
# Resolver fixtures
# ---------------------------------------------------------------------------

def generate_resolver_fixtures():
    """Generate TP/SL price and PnL computation fixtures."""
    out = ensure_dir(FIXTURES_DIR / "resolver")

    # TP/SL price computation cases
    tp_sl_cases = []
    for entry_price in [50000.0, 100.0, 0.5]:
        for is_long in [True, False]:
            pos = PositionType.LONG if is_long else PositionType.SHORT
            for tp_pct, sl_pct in [(3.0, 1.5), (1.0, 0.5), (10.0, 5.0), (0.2, 0.1)]:
                for fee_rate in [0.0005, 0.0002, 0.0]:
                    tp_price, sl_price = compute_tp_sl_prices(
                        entry_price, pos,
                        tp_pct=tp_pct, sl_pct=sl_pct,
                        taker_fee_rate=fee_rate,
                    )
                    tp_sl_cases.append({
                        "entry_price": entry_price,
                        "is_long": is_long,
                        "tp_pct": tp_pct,
                        "sl_pct": sl_pct,
                        "taker_fee_rate": fee_rate,
                        "tp_price_override": None,
                        "sl_price_override": None,
                        "expected_tp_price": tp_price,
                        "expected_sl_price": sl_price,
                    })

    # Override cases
    for entry_price in [50000.0]:
        for is_long in [True, False]:
            pos = PositionType.LONG if is_long else PositionType.SHORT
            tp_price, sl_price = compute_tp_sl_prices(
                entry_price, pos,
                tp_price_override=51500.0,
                sl_price_override=49000.0,
            )
            tp_sl_cases.append({
                "entry_price": entry_price,
                "is_long": is_long,
                "tp_pct": None,
                "sl_pct": None,
                "taker_fee_rate": 0.0005,
                "tp_price_override": 51500.0,
                "sl_price_override": 49000.0,
                "expected_tp_price": tp_price,
                "expected_sl_price": sl_price,
            })

    write_json(out / "tp_sl_prices.json", tp_sl_cases)

    # PnL computation cases
    pnl_cases = []
    for entry, exit_p in [(50000.0, 51500.0), (50000.0, 49000.0), (100.0, 103.0), (100.0, 98.0)]:
        for is_long in [True, False]:
            pos = PositionType.LONG if is_long else PositionType.SHORT
            for leverage in [1.0, 5.0, 10.0, 20.0]:
                for fee_rate in [0.0005, 0.0002, 0.0]:
                    net, gross, fee_drag = compute_pnl(
                        entry, exit_p, pos,
                        leverage=leverage,
                        taker_fee_rate=fee_rate,
                    )
                    pnl_cases.append({
                        "entry_price": entry,
                        "exit_price": exit_p,
                        "is_long": is_long,
                        "leverage": leverage,
                        "taker_fee_rate": fee_rate,
                        "expected_net_pnl_pct": net,
                        "expected_gross_pnl_pct": gross,
                        "expected_fee_drag_pct": fee_drag,
                    })

    write_json(out / "pnl_values.json", pnl_cases)


# ---------------------------------------------------------------------------
# Indicator fixtures
# ---------------------------------------------------------------------------

def generate_indicator_fixtures():
    """Generate indicator output fixtures from canonical BTC 1h data."""
    out = ensure_dir(FIXTURES_DIR / "indicators")

    client = BinanceClient()

    # Canonical 1000-bar BTC 1h window
    start = datetime(2024, 1, 1, tzinfo=UTC)
    end = start + timedelta(hours=1000)
    candles = client.fetch_klines("BTCUSDT", "1h", start, end)

    if not candles:
        print("  WARNING: no candle data fetched, skipping indicator fixtures")
        return

    import pandas as pd

    rows = []
    for c in candles:
        rows.append({
            "open_time": c.open_time,
            "close_time": c.close_time,
            "open": c.open,
            "high": c.high,
            "low": c.low,
            "close": c.close,
            "volume": c.volume,
            "taker_buy_volume": c.taker_buy_volume,
        })
    frame = pd.DataFrame(rows)

    # All indicators used across strategies
    all_indicators = (
        "rsi_14", "atr_14", "atr_ratio", "atr_72_avg",
        "ret_72h", "ret_24h", "ret_48h",
        "mom_slope", "ema_20",
        "bb_upper", "bb_lower", "bb_pct_b", "bb_width",
        "body", "body_ratio", "vol_ratio",
        "squeeze_on", "squeeze_count",
        "adx_14", "cvd", "t3",
        "kc_upper", "kc_lower",
        "vol_sma_20",
    )

    result = compute_indicator_frame(frame, all_indicators)

    # Serialize: convert timestamps to ISO, NaN to "NaN"
    records = []
    for _, row in result.iterrows():
        record = {}
        for col in result.columns:
            val = row[col]
            if isinstance(val, (pd.Timestamp, datetime)):
                record[col] = dt_to_iso(val)
            elif isinstance(val, float) and val != val:
                record[col] = "NaN"
            elif isinstance(val, (bool,)):
                record[col] = bool(val)
            else:
                record[col] = float(val) if isinstance(val, (int, float)) else str(val)
        records.append(record)

    write_json(out / "btc_1h_1000bars.json", {
        "symbol": "BTCUSDT",
        "interval": "1h",
        "start": dt_to_iso(start),
        "end": dt_to_iso(end),
        "bar_count": len(records),
        "indicators": list(all_indicators),
        "data": records,
    })

    # Warmup requirements
    warmup_cases = {}
    # Single indicators
    for ind in all_indicators:
        try:
            warmup_cases[ind] = required_warmup((ind,))
        except Exception:
            warmup_cases[ind] = None

    # Combo used by adaptive_regime_momentum
    arm_indicators = (
        "rsi_14", "atr_14", "atr_ratio", "ret_72h", "ret_24h",
        "squeeze_on", "squeeze_count", "mom_slope",
        "ema_20", "vol_ratio", "cvd", "body", "body_ratio",
    )
    warmup_cases["combo_adaptive_regime"] = required_warmup(arm_indicators)
    warmup_cases["combo_all"] = required_warmup(all_indicators)

    write_json(out / "warmup_requirements.json", warmup_cases)


# ---------------------------------------------------------------------------
# Engine fixtures (stats computation)
# ---------------------------------------------------------------------------

def generate_engine_fixtures():
    """Generate _compute_stats fixtures from synthetic trade data."""
    out = ensure_dir(FIXTURES_DIR / "engine")

    rng = Random(SEED)

    # Build a diverse set of synthetic TradeResults
    from backtester.models import TradeResult, ExitResolution, ResolutionLevel

    base_time = datetime(2024, 6, 1, tzinfo=UTC)
    trades = []

    # 30 trades: mix of TP, SL, TIMEOUT, UNFILLED, LONG, SHORT
    configs = [
        # (exit_reason, position_type, pnl_pct, size_mult)
        (ExitReason.TP, PositionType.LONG, 2.5, 1.0),
        (ExitReason.TP, PositionType.LONG, 1.8, 1.0),
        (ExitReason.SL, PositionType.LONG, -1.2, 1.0),
        (ExitReason.TP, PositionType.SHORT, 3.1, 1.0),
        (ExitReason.SL, PositionType.SHORT, -0.8, 1.0),
        (ExitReason.TIMEOUT, PositionType.LONG, 0.3, 1.0),
        (ExitReason.TIMEOUT, PositionType.SHORT, -0.5, 1.0),
        (ExitReason.UNFILLED, PositionType.LONG, 0.0, 1.0),
        (ExitReason.TP, PositionType.LONG, 4.2, 2.0),
        (ExitReason.SL, PositionType.LONG, -1.5, 0.5),
        (ExitReason.TP, PositionType.SHORT, 2.0, 1.0),
        (ExitReason.SL, PositionType.SHORT, -2.1, 1.0),
        (ExitReason.TP, PositionType.LONG, 1.1, 1.0),
        (ExitReason.SL, PositionType.LONG, -0.9, 1.0),
        (ExitReason.TP, PositionType.LONG, 5.0, 1.0),
        (ExitReason.SL, PositionType.SHORT, -1.8, 1.0),
        (ExitReason.TIMEOUT, PositionType.LONG, -0.2, 1.0),
        (ExitReason.TP, PositionType.SHORT, 1.5, 1.0),
        (ExitReason.SL, PositionType.LONG, -3.0, 1.0),
        (ExitReason.TP, PositionType.LONG, 2.8, 1.5),
        (ExitReason.UNFILLED, PositionType.SHORT, 0.0, 1.0),
        (ExitReason.TP, PositionType.LONG, 1.0, 1.0),
        (ExitReason.SL, PositionType.SHORT, -0.5, 1.0),
        (ExitReason.TP, PositionType.SHORT, 2.2, 1.0),
        (ExitReason.SL, PositionType.LONG, -1.0, 1.0),
        (ExitReason.TP, PositionType.LONG, 3.5, 1.0),
        (ExitReason.TIMEOUT, PositionType.SHORT, 0.1, 1.0),
        (ExitReason.SL, PositionType.LONG, -2.5, 1.0),
        (ExitReason.TP, PositionType.SHORT, 1.9, 1.0),
        (ExitReason.TP, PositionType.LONG, 0.8, 1.0),
    ]

    for i, (exit_reason, pos_type, pnl, size_mult) in enumerate(configs):
        entry_time = base_time + timedelta(hours=i * 4)
        exit_time = entry_time + timedelta(hours=rng.randint(1, 72))
        entry_price = 50000.0 + rng.uniform(-5000, 5000)

        signal = Signal(
            signal_date=entry_time - timedelta(seconds=15),
            position_type=pos_type,
            ticker="BTCUSDT" if i % 3 != 2 else "ETHUSDT",
            tp_pct=3.0,
            sl_pct=1.5,
            leverage=5.0,
            size_multiplier=size_mult,
        )

        gross_pnl = pnl + 0.05  # approximate fee drag
        fee_drag = 0.05

        trade = TradeResult(
            signal=signal,
            entry_price=entry_price,
            entry_time=entry_time,
            exit_price=entry_price * (1 + pnl / 100 / 5),
            exit_time=exit_time,
            exit_reason=exit_reason,
            resolution_level=ResolutionLevel.HOUR,
            tp_price=entry_price * 1.03,
            sl_price=entry_price * 0.985,
            pnl_pct=pnl,
            gross_pnl_pct=gross_pnl,
            fee_drag_pct=fee_drag,
        )
        trades.append(trade)

    # Compute stats
    stats = _compute_stats(trades)

    # Serialize trades + stats
    def trade_to_dict(t: TradeResult) -> dict:
        return {
            "signal": {
                "signal_date": dt_to_iso(t.signal.signal_date),
                "position_type": t.signal.position_type.value,
                "ticker": t.signal.ticker,
                "tp_pct": t.signal.tp_pct,
                "sl_pct": t.signal.sl_pct,
                "leverage": t.signal.leverage,
                "size_multiplier": t.signal.size_multiplier,
            },
            "entry_price": t.entry_price,
            "entry_time": dt_to_iso(t.entry_time),
            "exit_price": t.exit_price,
            "exit_time": dt_to_iso(t.exit_time),
            "exit_reason": t.exit_reason.value,
            "resolution_level": t.resolution_level.value,
            "tp_price": t.tp_price,
            "sl_price": t.sl_price,
            "pnl_pct": t.pnl_pct,
            "gross_pnl_pct": t.gross_pnl_pct,
            "fee_drag_pct": t.fee_drag_pct,
        }

    write_json(out / "compute_stats.json", {
        "trades": [trade_to_dict(t) for t in trades],
        "expected_stats": {
            "total_trades": stats["total_trades"],
            "wins": stats["wins"],
            "losses": stats["losses"],
            "open_trades": stats["open_trades"],
            "unfilled": stats["unfilled"],
            "win_rate": stats["win_rate"],
            "total_pnl_pct": stats["total_pnl_pct"],
            "avg_pnl_pct": stats["avg_pnl_pct"],
            "profit_factor": stats["profit_factor"],
            "max_drawdown_pct": stats["max_drawdown_pct"],
            "equity_curve": stats["equity_curve"],
        },
    })


# ---------------------------------------------------------------------------
# Calibration fixtures
# ---------------------------------------------------------------------------

def generate_calibration_fixtures():
    """Generate calibration grid search fixtures."""
    out = ensure_dir(FIXTURES_DIR / "calibration")

    from backtester.calibration import search_parameters
    import pandas as pd

    rng = Random(SEED)

    # Build a synthetic scoring frame
    n = 200
    frame = pd.DataFrame({
        "close": [50000 + rng.gauss(0, 500) for _ in range(n)],
        "rsi_14": [50 + rng.gauss(0, 15) for _ in range(n)],
        "atr_14": [500 + rng.gauss(0, 50) for _ in range(n)],
        "ret_72h": [rng.gauss(0, 3) for _ in range(n)],
    })

    # Simple score function: higher RSI threshold = fewer signals = lower score
    def score_fn(params, df):
        threshold = params["rsi_threshold"]
        matches = (df["rsi_14"] < threshold).sum()
        return float(matches)

    param_space = {
        "rsi_threshold": [20, 25, 30, 35, 40, 45, 50],
        "atr_mult": [1.0, 1.5, 2.0, 2.5],
    }

    result = search_parameters(param_space, score_fn, frame, max_workers=1)

    if result:
        write_json(out / "grid_search.json", {
            "param_space": param_space,
            "frame_rows": n,
            "expected_best_params": result.best_params,
            "expected_best_score": result.best_score,
            "expected_candidates_evaluated": result.candidates_evaluated,
        })
    else:
        write_json(out / "grid_search.json", {"result": None})


# ---------------------------------------------------------------------------
# Live state fixtures
# ---------------------------------------------------------------------------

def generate_live_fixtures():
    """Generate live state serialization fixtures."""
    out = ensure_dir(FIXTURES_DIR / "live")

    from live.models import (
        LivePosition, ExchangeOrder, PositionStatus,
        OrderSide, OrderType, OrderStatus,
    )

    base_time = datetime(2024, 6, 1, 12, 0, 0, tzinfo=UTC)

    signal = Signal(
        signal_date=base_time,
        position_type=PositionType.LONG,
        ticker="BTCUSDT",
        tp_pct=3.0,
        sl_pct=1.5,
        leverage=5.0,
    )

    entry_order = ExchangeOrder(
        order_id=12345,
        symbol="BTCUSDT",
        side=OrderSide.BUY,
        order_type=OrderType.MARKET,
        quantity=0.002,
        price=0.0,
        stop_price=0.0,
        status=OrderStatus.FILLED,
        filled_qty=0.002,
        avg_fill_price=50000.0,
        created_at=base_time,
        updated_at=base_time + timedelta(seconds=1),
    )

    tp_order = ExchangeOrder(
        order_id=12346,
        symbol="BTCUSDT",
        side=OrderSide.SELL,
        order_type=OrderType.TAKE_PROFIT_MARKET,
        quantity=0.002,
        price=0.0,
        stop_price=51550.0,
        status=OrderStatus.NEW,
    )

    sl_order = ExchangeOrder(
        order_id=12347,
        symbol="BTCUSDT",
        side=OrderSide.SELL,
        order_type=OrderType.STOP_MARKET,
        quantity=0.002,
        price=0.0,
        stop_price=49225.0,
        status=OrderStatus.NEW,
    )

    position = LivePosition(
        signal=signal,
        position_id="test-pos-001",
        strategy_id="TestStrategy",
        status=PositionStatus.OPEN,
        entry_order=entry_order,
        tp_order=tp_order,
        sl_order=sl_order,
        fill_price=50000.0,
        quantity=0.002,
        opened_at=base_time + timedelta(seconds=1),
    )

    # Use the tracker's serialization format
    from live.tracker import PositionTracker

    # Serialize position to dict (same format as live_state.json)
    pos_dict = PositionTracker._serialize_position(position)

    write_json(out / "position_state.json", {
        "positions": [pos_dict],
        "description": "Single OPEN LONG position with entry filled, TP/SL pending",
    })


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

SUBSETS = {
    "resolver": generate_resolver_fixtures,
    "indicators": generate_indicator_fixtures,
    "engine": generate_engine_fixtures,
    "calibration": generate_calibration_fixtures,
    "live": generate_live_fixtures,
}


def main():
    parser = argparse.ArgumentParser(description="Generate golden fixtures for Rust parity tests")
    parser.add_argument(
        "--subset", nargs="*", choices=list(SUBSETS.keys()),
        help="Which fixture subsets to generate (default: all)",
    )
    args = parser.parse_args()

    subsets = args.subset or list(SUBSETS.keys())

    print(f"Generating fixtures: {', '.join(subsets)}")
    print(f"Output directory: {FIXTURES_DIR}")
    print()

    for name in subsets:
        print(f"[{name}]")
        try:
            SUBSETS[name]()
        except Exception as e:
            print(f"  ERROR: {e}")
            import traceback
            traceback.print_exc()
        print()

    print("Done.")


if __name__ == "__main__":
    main()
