use exc::types::candle::{Period, PeriodKind};
use std::time::Duration;
use time::macros::offset;

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

/// Period to bar.
pub fn period_to_bar(period: &Period) -> Option<&'static str> {
    let utc_offset = period.utc_offset();
    let is_utc = utc_offset.is_utc();
    let is_hk = utc_offset == offset!(+8);

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
            _ => None,
        },
    }
}
