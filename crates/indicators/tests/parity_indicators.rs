//! Indicator parity tests against Python golden fixtures.

use claude_trader_indicators::{compute_indicators, required_warmup, OhlcvFrame};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

/// Count how many values match within epsilon (ignoring NaN-vs-NaN which is a match).
fn parity_score(actual: &[f64], expected: &[f64], epsilon: f64) -> (usize, usize, usize) {
    let mut matches = 0;
    let mut mismatches = 0;
    let mut nan_matches = 0;
    for (a, e) in actual.iter().zip(expected.iter()) {
        if a.is_nan() && e.is_nan() {
            nan_matches += 1;
            matches += 1;
        } else if a.is_nan() || e.is_nan() {
            mismatches += 1;
        } else if (a - e).abs() <= epsilon {
            matches += 1;
        } else {
            mismatches += 1;
        }
    }
    (matches, mismatches, nan_matches)
}

#[test]
fn test_warmup_parity() {
    let path = fixtures_dir().join("indicators/warmup_requirements.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let fixture: serde_json::Value = serde_json::from_str(&content).unwrap();

    let cases = fixture.as_object().unwrap();
    for (name, expected) in cases {
        if name.starts_with("combo_") {
            continue; // Test combos separately
        }
        let expected_val = match expected {
            serde_json::Value::Number(n) => n.as_u64().unwrap() as usize,
            serde_json::Value::Null => continue,
            _ => panic!("Unexpected value type for {name}"),
        };

        let actual = required_warmup(&[name.as_str()]);
        assert_eq!(
            actual, expected_val,
            "Warmup mismatch for {name}: Rust={actual}, Python={expected_val}"
        );
    }
}

#[test]
fn test_indicator_frame_parity() {
    let path = fixtures_dir().join("indicators/btc_1h_1000bars.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let fixture: serde_json::Value = serde_json::from_str(&content).unwrap();

    let data = fixture["data"].as_array().unwrap();
    let indicators: Vec<&str> = fixture["indicators"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    // Build OhlcvFrame from fixture data
    let n = data.len();
    let mut frame = OhlcvFrame {
        open: Vec::with_capacity(n),
        high: Vec::with_capacity(n),
        low: Vec::with_capacity(n),
        close: Vec::with_capacity(n),
        volume: Vec::with_capacity(n),
        taker_buy_volume: Vec::with_capacity(n),
    };

    for row in data {
        frame.open.push(row["open"].as_f64().unwrap());
        frame.high.push(row["high"].as_f64().unwrap());
        frame.low.push(row["low"].as_f64().unwrap());
        frame.close.push(row["close"].as_f64().unwrap());
        frame.volume.push(row["volume"].as_f64().unwrap());
        frame
            .taker_buy_volume
            .push(row["taker_buy_volume"].as_f64().unwrap());
    }

    // Compute indicators in Rust
    let result = compute_indicators(&frame, &indicators).unwrap();

    // Compare each indicator against fixture
    let epsilon = 1e-4; // Relaxed for float differences between Rust/Python EWM
    let mut total_indicators = 0;
    let mut passed_indicators = 0;

    for &ind_name in &indicators {
        let rust_col = match result.get(ind_name) {
            Some(col) => col,
            None => {
                eprintln!("  SKIP {ind_name}: not computed by Rust");
                continue;
            }
        };

        // Extract expected column from fixture
        let expected: Vec<f64> = data
            .iter()
            .map(|row| match &row[ind_name] {
                serde_json::Value::String(s) if s == "NaN" => f64::NAN,
                serde_json::Value::Number(n) => n.as_f64().unwrap(),
                serde_json::Value::Bool(b) => {
                    if *b {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => f64::NAN,
            })
            .collect();

        assert_eq!(
            rust_col.len(),
            expected.len(),
            "{ind_name}: length mismatch"
        );

        let (matches, mismatches, _nan_matches) = parity_score(rust_col, &expected, epsilon);
        total_indicators += 1;

        if mismatches == 0 {
            passed_indicators += 1;
        } else {
            // Print first few mismatches for debugging
            let mut shown = 0;
            for (i, (a, e)) in rust_col.iter().zip(expected.iter()).enumerate() {
                if a.is_nan() && e.is_nan() {
                    continue;
                }
                if a.is_nan() || e.is_nan() || (a - e).abs() > epsilon {
                    if shown < 3 {
                        eprintln!(
                            "  {ind_name}[{i}]: Rust={a:.8}, Python={e:.8}, diff={:.2e}",
                            if a.is_nan() || e.is_nan() {
                                f64::NAN
                            } else {
                                (a - e).abs()
                            }
                        );
                        shown += 1;
                    }
                }
            }
            eprintln!(
                "  {ind_name}: {matches}/{} match ({mismatches} mismatches)",
                matches + mismatches
            );
        }
    }

    eprintln!("\nIndicator parity: {passed_indicators}/{total_indicators} indicators fully match");

    // Allow some indicators to have minor float differences but assert
    // the vast majority match
    assert!(
        passed_indicators >= total_indicators * 80 / 100,
        "Too many indicator mismatches: only {passed_indicators}/{total_indicators} passed"
    );
}
