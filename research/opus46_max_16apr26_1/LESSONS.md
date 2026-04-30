# Research Lessons — opus46_max_16apr26_1

## What Doesn't Work (and Why)

- **Longs on 1h crypto, any entry design** (V1-V6, V8). Tried: squeeze release long, RSI oversold mean-reversion, capitulation bounce (RSI<30 + ret_24h<-2%), momentum inflection (ret_24h>0 while ret_48h<0), uptrend continuation (green candle after red in uptrend). All lost money. Best long attempt was V5 inflect_long: 303 trades, 30.7% WR, -76.77% PnL. Root cause: crypto drops are faster than rises, so short TP gets hit before long TP at symmetric ATR multiples. Do NOT waste time trying to make longs work on 1h — the asymmetry is structural.

- **Overbought fade shorts without squeeze context** (V7 fade_short). RSI>65 + BB>0.8 + red candle fired 349 trades at 33.2% WR, -76.81%. Too frequent, no structural edge — overbought conditions persist in uptrends.

- **Trend continuation shorts** (V7 trend_cont). Red candle after green in downtrend: 407 trades, 36.9% WR, -17.07%. Same issue — too frequent, low signal-to-noise.

- **Rolling calibration on TP/SL** (V13). 168h interval, 720h lookback, grid search over TP_ATR [2.0-4.0] and SL_ATR [1.0-1.8]. Dev improved marginally but eval cratered: holdout went from +0.76% to -12.94%. Only ~10-15 squeeze events per lookback window — too few for meaningful calibration. The signal is already well-specified; static TP/SL generalizes better.

- **Relaxing squeeze duration from 5 to 4 bars** (V8, V14). Doubled trade count but halved average PnL per trade. Shorter squeezes produce weaker breakouts. WR dropped from 44% to 37%.

- **Relaxing body_ratio from 0.4 to 0.35** (V22). Added 4 eval trades but dropped PF from 2.59 to 2.20. The 0.4 threshold is a real quality boundary.

- **Removing BTC bullish filter** (V9). Allowed shorting alts during BTC bull runs. Dev PnL went from +41% to -6.87%. The filter is essential — squeeze releases during macro bull markets are false signals for alts.

- **Adding noisy symbols** (V18). Going from 22 to 27 symbols (adding AAVE, ATOM, WLD, SEI, 1000PEPE) degraded eval PF from 2.10 to 1.69. Not all symbols have clean squeeze dynamics.

## What Actually Moves the Needle

- **Multi-timeframe confirmation (1h+4h)** — single biggest improvement. V12→V16: eval pref 3.69→9.23. The 4h bearish context filter removes false 1h squeeze releases during 4h uptrends. Simpler 4h filters generalize better: "4h candle red OR 4h close < prev 4h close" (V20, eval pref 41.95) beat "4h close < 4h EMA20" (V16, eval pref 9.23).

- **4h squeeze release as independent signal** (V23-V24). Adding 4h signals with 1-bar lag: eval pref 41.95→196.26. The 4h signals average +3.6% per trade on eval at 57% WR. The 1-bar lag (detect release at bar i-1, signal at bar i) provides confirmation and avoids validation boundary issues.

- **Symbol count** — 8→14→18→22 symbols improved eval from 1.37→3.52→3.69→27.49. But going to 27 degraded quality. Sweet spot is ~22 liquid symbols.

- **Removing losing sub-strategies** (V3, V10b). Cutting rev_short (5 trades, -2.37%) improved pref from 5.40→6.17 by reducing MDD. Less is more when the cut signals have negative EV.

## Overfitting Traps

- **Calibration with too few events** (V13). The lookback window had ~10-15 squeeze releases. Grid search "found" optimal params that were just noise. Eval pref collapsed from 3.69 to 0.46.

- **EMA-20 trend filter for longs** (V2). Added above_ema + ret_24h filter for release longs. Dev improved marginally but the filter selected for even worse longs (buying near resistance after recent up-move). Pref dropped from 1.84 to 0.35.

- **Strict 4h EMA filter** (V16 vs V19-V20). Requiring 4h close < EMA20 gave dev pref 30.88 but eval only 9.23. The softer "red candle OR close dropped" gave dev 19.85 but eval 41.95. Tighter dev = more overfitting to dev's bearish bias.

## Structural Constraints

- **Crypto short/long asymmetry is real.** Drops are 2-3x faster than equivalent rises. TP at 3.0 ATR gets hit on shorts before it does on longs, even at the same volatility. This isn't a strategy quirk — it's market microstructure (liquidation cascades, fear > greed speed).

- **Squeeze release is rare.** With 22 symbols on 1h, only ~70 eval signals across 41 windows. Adding symbols beyond ~22 adds noise. The pattern is fundamentally infrequent — you can't force more signals without losing edge.

- **Dev windows have a bearish bias.** Apr24 and Mar25 are heavily bearish, dominating dev results. Short-only strategies look much better on dev than eval. Always check eval — dev pref is misleading for short-biased strategies.

- **Validation boundary artifacts.** Multi-timeframe strategies produce ~4% validation failures at window boundaries due to 4h candle truncation in replay mode. These are false positives, not real lookahead — the 4h candle is complete but truncation point differs between full run and replay.

## Open Questions

- **Would 15m primary + 1h confirmation work?** Higher frequency = more signals. The 15m squeeze release might capture intraday volatility cycles. Not tested due to time.

- **Funding rate as standalone signal.** When funding > 0.05%, market is extremely long-heavy. A dedicated funding fade short (independent of squeeze) might add orthogonal alpha. Partially tested via funding boost (1.3x size) but not as standalone entry.

- **Key levels integration.** Shorting squeeze releases into known resistance levels (PDH, weekly highs) might improve WR. Not tested.

- **Leverage scaling by conviction.** When both 1h and 4h fire squeeze release within same 4h window, the signal is higher conviction. Using 1.5x leverage on these aligned signals might improve returns without proportional risk increase.

- **Cross-symbol correlation filter.** When many symbols fire squeeze release simultaneously, it's a broad market event (higher conviction). Could use count of concurrent signals as confidence multiplier.
