use exc_core::types::candle::{Period, PeriodKind};
use std::time::Duration;
use time::{macros::offset, UtcOffset};

const W1: Duration = Duration::from_secs(7 * 24 * 3600);
const D3: Duration = Duration::from_secs(3 * 24 * 3600);
const D2: Duration = Duration::from_secs(2 * 24 * 3600);
const D1: Duration = Duration::from_secs(24 * 3600);
const H12: Duration = Duration::from_secs(12 * 3600);
const H6: Duration = Duration::from_secs(6 * 3600);
const H4: Duration = Duration::from_secs(4 * 3600);
const H2: Duration = Duration::from_secs(2 * 3600);
const H1: Duration = Duration::from_secs(3600);
const M30: Duration = Duration::from_secs(1800);
const M15: Duration = Duration::from_secs(900);
const M5: Duration = Duration::from_secs(300);
const M3: Duration = Duration::from_secs(180);
const M1: Duration = Duration::from_secs(60);
const S1: Duration = Duration::from_secs(1);

const HK: UtcOffset = offset!(+8);

/// Period to bar.
pub fn period_to_bar(period: &Period) -> Option<&'static str> {
    let utc_offset = period.utc_offset();
    let is_utc = utc_offset.is_utc();
    let is_hk = utc_offset == HK;

    match period.kind() {
        PeriodKind::Year => {
            if is_utc {
                Some("1Yutc")
            } else if is_hk {
                Some("1Y")
            } else {
                None
            }
        }
        PeriodKind::Month => {
            if is_utc {
                Some("1Mutc")
            } else if is_hk {
                Some("1M")
            } else {
                None
            }
        }
        PeriodKind::Duration(dur) => match dur {
            W1 => {
                if is_utc {
                    Some("1Wutc")
                } else if is_hk {
                    Some("1W")
                } else {
                    None
                }
            }
            D3 => {
                if is_utc {
                    Some("3Dutc")
                } else if is_hk {
                    Some("3D")
                } else {
                    None
                }
            }
            D2 => {
                if is_utc {
                    Some("2Dutc")
                } else if is_hk {
                    Some("2D")
                } else {
                    None
                }
            }
            D1 => {
                if is_utc {
                    Some("1Dutc")
                } else if is_hk {
                    Some("1D")
                } else {
                    None
                }
            }
            H12 => {
                if is_utc {
                    Some("12Hutc")
                } else if is_hk {
                    Some("12H")
                } else {
                    None
                }
            }
            H6 => {
                if is_utc {
                    Some("6Hutc")
                } else if is_hk {
                    Some("6H")
                } else {
                    None
                }
            }
            H4 => Some("4H"),
            H2 => Some("2H"),
            H1 => Some("1H"),
            M30 => Some("30m"),
            M15 => Some("15m"),
            M5 => Some("5m"),
            M3 => Some("3m"),
            M1 => Some("1m"),
            S1 => Some("1s"),
            _ => None,
        },
    }
}

/// Bar to period.
pub fn bar_to_period(bar: &str) -> Option<Period> {
    match bar {
        "1s" => Some(Period::seconds(HK, 1)),
        "1m" => Some(Period::minutes(HK, 1)),
        "3m" => Some(Period::minutes(HK, 3)),
        "5m" => Some(Period::minutes(HK, 5)),
        "15m" => Some(Period::minutes(HK, 15)),
        "30m" => Some(Period::minutes(HK, 30)),
        "1H" => Some(Period::hours(HK, 1)),
        "2H" => Some(Period::hours(HK, 2)),
        "4H" => Some(Period::hours(HK, 4)),
        "6H" => Some(Period::hours(HK, 6)),
        "6Hutc" => Some(Period::hours(UtcOffset::UTC, 6)),
        "12H" => Some(Period::hours(HK, 12)),
        "12Hutc" => Some(Period::hours(UtcOffset::UTC, 12)),
        "1D" => Some(Period::day(HK)),
        "1Dutc" => Some(Period::day(UtcOffset::UTC)),
        "3D" => Some(Period::days(HK, 3)),
        "3Dutc" => Some(Period::days(UtcOffset::UTC, 3)),
        "1W" => Some(Period::weeks(HK, 1)),
        "1Wutc" => Some(Period::weeks(UtcOffset::UTC, 1)),
        "1M" => Some(Period::month(HK)),
        "1Mutc" => Some(Period::month(UtcOffset::UTC)),
        "1Y" => Some(Period::year(HK)),
        "1Yutc" => Some(Period::year(UtcOffset::UTC)),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_bar(bar: &str) {
        let period = bar_to_period(bar).unwrap();
        let out_bar = period_to_bar(&period).unwrap();
        assert_eq!(bar, out_bar);
    }
    fn test_period(period: Period) {
        let bar = period_to_bar(&period).unwrap();
        let out_period = bar_to_period(bar).unwrap();
        assert_eq!(period, out_period);
    }

    #[test]
    fn test_all_bar() {
        for bar in [
            "1s", "1m", "3m", "5m", "15m", "30m", "1H", "2H", "4H", "6H", "6Hutc", "12H", "12Hutc",
            "1D", "1Dutc", "3D", "3Dutc", "1W", "1Wutc", "1M", "1Mutc", "1Y", "1Yutc",
        ] {
            test_bar(bar);
        }
    }
    #[test]
    fn test_all_period() {
        for period in [
            W1, D3, D2, D1, H12, H6, H4, H2, H1, M30, M15, M5, M3, M1, S1,
        ] {
            let period = Period {
                offset: HK,
                kind: PeriodKind::Duration(period),
            };
            test_period(period);
        }
    }
}
