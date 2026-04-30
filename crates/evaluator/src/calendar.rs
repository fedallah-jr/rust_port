//! Eval window calendar — hardcoded window definitions matching Python.

use chrono::{Duration, TimeZone, Utc};
use claude_trader_models::EvalWindow;

fn dt(y: i32, m: u32, d: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap()
}

fn weekly_block(
    prefix: &str,
    category: &str,
    start: chrono::DateTime<Utc>,
    count: usize,
) -> Vec<EvalWindow> {
    let mut windows = Vec::new();
    let mut cursor = start;
    for idx in 1..=count {
        let end = cursor + Duration::days(7);
        windows.push(EvalWindow {
            name: format!("{prefix}_W{idx}"),
            category: category.to_string(),
            start: cursor,
            end,
        });
        cursor = end;
    }
    windows
}

pub fn development_windows() -> Vec<EvalWindow> {
    let mut w = Vec::new();

    // Core development blocks — regime classified by BTC price action
    w.extend(weekly_block("Apr24", "dev [bearish]", dt(2024, 4, 15), 4));
    w.extend(weekly_block("Aug24", "dev [sideways]", dt(2024, 8, 1), 4));
    w.extend(weekly_block("Oct24", "dev [bullish]", dt(2024, 10, 15), 4));
    w.extend(weekly_block("Dec24", "dev [bullish]", dt(2024, 12, 1), 4));
    w.extend(weekly_block("JanFeb25", "dev [sideways]", dt(2025, 1, 15), 4));
    w.extend(weekly_block("Mar25", "dev [bearish]", dt(2025, 3, 1), 4));

    // Stress windows — bullish
    w.push(EvalWindow {
        name: "DEVX_MAY24_BULL".into(),
        start: dt(2024, 5, 14),
        end: dt(2024, 5, 21),
        category: "dev_stress [bullish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_NOV24_BULL".into(),
        start: dt(2024, 11, 15),
        end: dt(2024, 11, 22),
        category: "dev_stress [bullish]".into(),
    });
    // Stress windows — bearish
    w.push(EvalWindow {
        name: "DEVX_FEB25_BEAR".into(),
        start: dt(2025, 2, 20),
        end: dt(2025, 2, 27),
        category: "dev_stress [bearish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_AUG23_CRASH".into(),
        start: dt(2023, 8, 12),
        end: dt(2023, 8, 19),
        category: "dev_stress [bearish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_MAR24_HV".into(),
        start: dt(2024, 3, 16),
        end: dt(2024, 3, 23),
        category: "dev_stress [bearish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_JUN24_CRASH".into(),
        start: dt(2024, 6, 29),
        end: dt(2024, 7, 6),
        category: "dev_stress [bearish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_SEP24_BEAR".into(),
        start: dt(2024, 9, 28),
        end: dt(2024, 10, 5),
        category: "dev_stress [bearish]".into(),
    });
    // Stress windows — sideways
    w.push(EvalWindow {
        name: "DEVX_AUG25_CHOP".into(),
        start: dt(2025, 8, 20),
        end: dt(2025, 8, 27),
        category: "dev_stress [sideways]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_DEC23_DISP".into(),
        start: dt(2023, 12, 9),
        end: dt(2023, 12, 16),
        category: "dev_stress [sideways]".into(),
    });
    w.push(EvalWindow {
        name: "DEVX_JAN24_CHOP".into(),
        start: dt(2024, 1, 6),
        end: dt(2024, 1, 13),
        category: "dev_stress [sideways]".into(),
    });

    // Bull regime windows
    w.push(EvalWindow {
        name: "DEVB_NOV20_BREAKOUT".into(),
        start: dt(2020, 11, 18),
        end: dt(2020, 11, 25),
        category: "dev_bull [bullish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVB_JAN21_MANIA".into(),
        start: dt(2021, 1, 28),
        end: dt(2021, 2, 4),
        category: "dev_bull [bullish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVB_AUG21_REBOUND".into(),
        start: dt(2021, 8, 9),
        end: dt(2021, 8, 16),
        category: "dev_bull [bullish]".into(),
    });
    w.push(EvalWindow {
        name: "DEVB_JUL22_RALLY".into(),
        start: dt(2022, 7, 13),
        end: dt(2022, 7, 20),
        category: "dev_bull [bullish]".into(),
    });

    // Pairs windows — by regime
    w.extend(weekly_block(
        "DEVP_QUIET23",
        "dev_pairs [sideways]",
        dt(2023, 9, 9),
        2,
    ));
    w.extend(weekly_block(
        "DEVP_MAR24_REVERSAL",
        "dev_pairs [sideways]",
        dt(2024, 3, 23),
        2,
    ));
    w.extend(weekly_block(
        "DEVP_JUL25_TRUMP_BULL",
        "dev_pairs [bullish]",
        dt(2025, 7, 5),
        2,
    ));
    w.extend(weekly_block(
        "DEVP_DEC25_BEAR",
        "dev_pairs [bearish]",
        dt(2025, 12, 13),
        2,
    ));

    // Random windows — by regime
    w.extend(weekly_block(
        "DEVR_FEB23",
        "dev_random [sideways]",
        dt(2023, 2, 5),
        3,
    ));
    w.extend(weekly_block(
        "DEVR_SEP23",
        "dev_random [sideways]",
        dt(2023, 9, 24),
        3,
    ));
    w.extend(weekly_block(
        "DEVR_JUN24",
        "dev_random [sideways]",
        dt(2024, 6, 2),
        3,
    ));
    w.extend(weekly_block(
        "DEVR_AUG25",
        "dev_random [sideways]",
        dt(2025, 8, 31),
        3,
    ));

    w
}

pub fn evaluation_windows() -> Vec<EvalWindow> {
    let mut w = Vec::new();

    w.extend(weekly_block("Apr25", "holdout", dt(2025, 4, 1), 4));
    w.extend(weekly_block("OOS26", "holdout", dt(2026, 3, 1), 3));

    w.push(EvalWindow {
        name: "OOS2_BULL25".into(),
        start: dt(2025, 5, 6),
        end: dt(2025, 5, 13),
        category: "oos2".into(),
    });
    w.push(EvalWindow {
        name: "OOS2_BEAR25".into(),
        start: dt(2025, 10, 3),
        end: dt(2025, 10, 10),
        category: "oos2".into(),
    });
    w.push(EvalWindow {
        name: "OOS2_CHOP25".into(),
        start: dt(2025, 11, 27),
        end: dt(2025, 12, 4),
        category: "oos2".into(),
    });
    w.push(EvalWindow {
        name: "OOS2_CAPIT26".into(),
        start: dt(2026, 1, 29),
        end: dt(2026, 2, 5),
        category: "oos2".into(),
    });

    w.push(EvalWindow {
        name: "OOS3_JUL23_DISP".into(),
        start: dt(2023, 7, 8),
        end: dt(2023, 7, 15),
        category: "oos3".into(),
    });
    w.push(EvalWindow {
        name: "OOS3_NOV23_DISP".into(),
        start: dt(2023, 11, 11),
        end: dt(2023, 11, 18),
        category: "oos3".into(),
    });
    w.push(EvalWindow {
        name: "OOS3_JUL24_CRASH".into(),
        start: dt(2024, 7, 20),
        end: dt(2024, 7, 27),
        category: "oos3".into(),
    });
    w.push(EvalWindow {
        name: "OOS3_AUG24_BEAR".into(),
        start: dt(2024, 8, 31),
        end: dt(2024, 9, 7),
        category: "oos3".into(),
    });
    w.push(EvalWindow {
        name: "OOS3_OCT25_HV".into(),
        start: dt(2025, 10, 11),
        end: dt(2025, 10, 18),
        category: "oos3".into(),
    });
    w.push(EvalWindow {
        name: "OOS3_NOV25_CRASH".into(),
        start: dt(2025, 11, 15),
        end: dt(2025, 11, 22),
        category: "oos3".into(),
    });

    w.push(EvalWindow {
        name: "OOS4_JAN21_BREAKOUT".into(),
        start: dt(2021, 1, 1),
        end: dt(2021, 1, 8),
        category: "oos4".into(),
    });
    w.push(EvalWindow {
        name: "OOS4_FEB21_MANIA".into(),
        start: dt(2021, 2, 4),
        end: dt(2021, 2, 11),
        category: "oos4".into(),
    });
    w.push(EvalWindow {
        name: "OOS4_APR21_ALT".into(),
        start: dt(2021, 4, 9),
        end: dt(2021, 4, 16),
        category: "oos4".into(),
    });
    w.push(EvalWindow {
        name: "OOS4_JAN23_REPRICING".into(),
        start: dt(2023, 1, 8),
        end: dt(2023, 1, 15),
        category: "oos4".into(),
    });

    w.extend(weekly_block("OOS5_OCT23_BULL", "oos5", dt(2023, 10, 21), 2));
    w.extend(weekly_block(
        "OOS5_AUG25_TRUMP_BULL",
        "oos5",
        dt(2025, 8, 2),
        2,
    ));
    w.extend(weekly_block("OOS5_NOV25_BEAR", "oos5", dt(2025, 11, 1), 2));
    w.extend(weekly_block(
        "OOS5_FEB26_FLAT_BEAR",
        "oos5",
        dt(2026, 2, 14),
        2,
    ));

    w.extend(weekly_block(
        "EVALR_MAY23",
        "evaluation_random",
        dt(2023, 5, 21),
        3,
    ));
    w.extend(weekly_block(
        "EVALR_FEB24",
        "evaluation_random",
        dt(2024, 2, 18),
        3,
    ));
    w.extend(weekly_block(
        "EVALR_JUN25",
        "evaluation_random",
        dt(2025, 6, 1),
        3,
    ));
    w.extend(weekly_block(
        "EVALR_JAN26",
        "evaluation_random",
        dt(2026, 1, 4),
        3,
    ));

    w
}

pub fn all_windows() -> Vec<EvalWindow> {
    let mut w = development_windows();
    w.extend(evaluation_windows());
    w
}

pub fn test_windows() -> Vec<EvalWindow> {
    vec![
        EvalWindow {
            name: "TEST_W1".into(),
            start: dt(2023, 1, 1),
            end: dt(2023, 1, 8),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W2".into(),
            start: dt(2023, 1, 15),
            end: dt(2023, 1, 22),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W3".into(),
            start: dt(2023, 1, 22),
            end: dt(2023, 1, 29),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W4".into(),
            start: dt(2023, 1, 29),
            end: dt(2023, 2, 5),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W5".into(),
            start: dt(2023, 2, 26),
            end: dt(2023, 3, 5),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W6".into(),
            start: dt(2023, 3, 5),
            end: dt(2023, 3, 12),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W7".into(),
            start: dt(2023, 3, 12),
            end: dt(2023, 3, 19),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W8".into(),
            start: dt(2023, 3, 19),
            end: dt(2023, 3, 26),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W9".into(),
            start: dt(2023, 3, 26),
            end: dt(2023, 4, 2),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W10".into(),
            start: dt(2023, 4, 2),
            end: dt(2023, 4, 9),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W11".into(),
            start: dt(2023, 4, 9),
            end: dt(2023, 4, 16),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W12".into(),
            start: dt(2023, 4, 16),
            end: dt(2023, 4, 23),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W13".into(),
            start: dt(2023, 4, 23),
            end: dt(2023, 4, 30),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W14".into(),
            start: dt(2023, 4, 30),
            end: dt(2023, 5, 7),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W15".into(),
            start: dt(2023, 5, 7),
            end: dt(2023, 5, 14),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W16".into(),
            start: dt(2023, 5, 14),
            end: dt(2023, 5, 21),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W17".into(),
            start: dt(2023, 6, 11),
            end: dt(2023, 6, 18),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W18".into(),
            start: dt(2023, 6, 18),
            end: dt(2023, 6, 25),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W19".into(),
            start: dt(2023, 6, 25),
            end: dt(2023, 7, 2),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W20".into(),
            start: dt(2023, 7, 15),
            end: dt(2023, 7, 22),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W21".into(),
            start: dt(2023, 7, 22),
            end: dt(2023, 7, 29),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W22".into(),
            start: dt(2023, 7, 29),
            end: dt(2023, 8, 5),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W23".into(),
            start: dt(2023, 8, 5),
            end: dt(2023, 8, 12),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W24".into(),
            start: dt(2023, 8, 19),
            end: dt(2023, 8, 26),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W25".into(),
            start: dt(2023, 8, 26),
            end: dt(2023, 9, 2),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W26".into(),
            start: dt(2023, 9, 2),
            end: dt(2023, 9, 9),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W27".into(),
            start: dt(2023, 11, 4),
            end: dt(2023, 11, 11),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W28".into(),
            start: dt(2023, 11, 18),
            end: dt(2023, 11, 25),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W29".into(),
            start: dt(2023, 11, 25),
            end: dt(2023, 12, 2),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W30".into(),
            start: dt(2023, 12, 2),
            end: dt(2023, 12, 9),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W31".into(),
            start: dt(2023, 12, 16),
            end: dt(2023, 12, 23),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W32".into(),
            start: dt(2023, 12, 23),
            end: dt(2023, 12, 30),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W33".into(),
            start: dt(2023, 12, 30),
            end: dt(2024, 1, 6),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W34".into(),
            start: dt(2024, 1, 13),
            end: dt(2024, 1, 20),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W35".into(),
            start: dt(2024, 1, 20),
            end: dt(2024, 1, 27),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W36".into(),
            start: dt(2024, 1, 27),
            end: dt(2024, 2, 3),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W37".into(),
            start: dt(2024, 2, 3),
            end: dt(2024, 2, 10),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W38".into(),
            start: dt(2024, 2, 10),
            end: dt(2024, 2, 17),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W39".into(),
            start: dt(2024, 4, 6),
            end: dt(2024, 4, 13),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W40".into(),
            start: dt(2024, 5, 21),
            end: dt(2024, 5, 28),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W41".into(),
            start: dt(2024, 7, 6),
            end: dt(2024, 7, 13),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W42".into(),
            start: dt(2024, 7, 13),
            end: dt(2024, 7, 20),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W43".into(),
            start: dt(2024, 9, 7),
            end: dt(2024, 9, 14),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W44".into(),
            start: dt(2024, 9, 14),
            end: dt(2024, 9, 21),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W45".into(),
            start: dt(2024, 9, 21),
            end: dt(2024, 9, 28),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W46".into(),
            start: dt(2024, 10, 5),
            end: dt(2024, 10, 12),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W47".into(),
            start: dt(2024, 11, 22),
            end: dt(2024, 11, 29),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W48".into(),
            start: dt(2024, 12, 29),
            end: dt(2025, 1, 5),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W49".into(),
            start: dt(2025, 1, 5),
            end: dt(2025, 1, 12),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W50".into(),
            start: dt(2025, 2, 12),
            end: dt(2025, 2, 19),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W51".into(),
            start: dt(2025, 4, 29),
            end: dt(2025, 5, 6),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W52".into(),
            start: dt(2025, 5, 13),
            end: dt(2025, 5, 20),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W53".into(),
            start: dt(2025, 5, 20),
            end: dt(2025, 5, 27),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W54".into(),
            start: dt(2025, 6, 22),
            end: dt(2025, 6, 29),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W55".into(),
            start: dt(2025, 7, 19),
            end: dt(2025, 7, 26),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W56".into(),
            start: dt(2025, 7, 26),
            end: dt(2025, 8, 2),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W57".into(),
            start: dt(2025, 9, 21),
            end: dt(2025, 9, 28),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W58".into(),
            start: dt(2025, 10, 18),
            end: dt(2025, 10, 25),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W59".into(),
            start: dt(2025, 10, 25),
            end: dt(2025, 11, 1),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W60".into(),
            start: dt(2025, 12, 4),
            end: dt(2025, 12, 11),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W61".into(),
            start: dt(2025, 12, 27),
            end: dt(2026, 1, 3),
            category: "test".into(),
        },
        EvalWindow {
            name: "TEST_W62".into(),
            start: dt(2026, 2, 5),
            end: dt(2026, 2, 12),
            category: "test".into(),
        },
    ]
}

pub fn complete_windows() -> Vec<EvalWindow> {
    let mut w = all_windows();
    w.extend(test_windows());
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_counts() {
        assert_eq!(development_windows().len(), 58);
        assert_eq!(evaluation_windows().len(), 41);
        assert_eq!(all_windows().len(), 99);
        assert_eq!(test_windows().len(), 62);
        assert_eq!(complete_windows().len(), 161);
    }

    #[test]
    fn test_calendar_spot_checks() {
        let eval = evaluation_windows();
        assert_eq!(eval.first().unwrap().name, "Apr25_W1");
        assert_eq!(eval.last().unwrap().name, "EVALR_JAN26_W3");

        let complete = complete_windows();
        assert_eq!(complete.first().unwrap().name, "Apr24_W1");
        assert_eq!(complete.last().unwrap().name, "TEST_W62");
    }
}
