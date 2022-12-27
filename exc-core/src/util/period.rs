use std::{
    cmp::Ordering,
    ops::{Bound, RangeBounds},
    time::Duration,
};

use indicator::{window::mode::tumbling::period::PeriodKind, Period};
use time::{macros::time, Date, Month, OffsetDateTime, PrimitiveDateTime};

const WEEK_OFFSET: Duration = Duration::from_secs(4 * 24 * 3600);

/// Truncate the ts.
/// # Example
/// ```
/// use exc::util::trunc;
/// use time::macros::datetime;
/// use std::time::Duration;
///
/// let ts = datetime!(2022-05-06 12:31:59 +08:00);
/// assert_eq!(
///     trunc(ts, Duration::from_secs(7 * 24 * 3600)).unwrap(),
///     datetime!(2022-05-02 00:00:00 +08:00),
/// );
/// ```
pub fn trunc(ts: OffsetDateTime, duration: Duration) -> Option<OffsetDateTime> {
    let span = duration.as_nanos();
    if span > i64::MAX as u128 {
        return None;
    }
    let span = span as i64;
    let base = OffsetDateTime::UNIX_EPOCH.replace_offset(ts.offset()) + WEEK_OFFSET;
    let stamp = (ts - base).whole_nanoseconds();
    if span as i128 > stamp.abs() {
        return None;
    }
    let delta_down = (stamp % (span as i128)) as i64;
    match delta_down.cmp(&0) {
        Ordering::Equal => Some(ts),
        Ordering::Greater => Some(ts - time::Duration::nanoseconds(delta_down)),
        Ordering::Less => Some(ts - time::Duration::nanoseconds(span - delta_down.abs())),
    }
}

/// A range iterator of [`OffsetDateTime`].
pub struct RangeIter {
    period: PeriodKind,
    end: Bound<OffsetDateTime>,
    current: Bound<OffsetDateTime>,
}

impl RangeIter {
    fn current_included(&mut self) -> Option<OffsetDateTime> {
        let current = match self.current {
            Bound::Unbounded => None,
            Bound::Included(ts) => match self.period {
                PeriodKind::Year => ts
                    .replace_time(time!(00:00:00))
                    .replace_day(1)
                    .ok()?
                    .replace_month(Month::January)
                    .ok(),
                PeriodKind::Month => ts.replace_time(time!(00:00:00)).replace_day(1).ok(),
                PeriodKind::Duration(dur) => trunc(ts, dur),
            },
            Bound::Excluded(ts) => match self.period {
                PeriodKind::Year => {
                    let year = ts.year() + 1;
                    Some(ts.replace_date_time(PrimitiveDateTime::new(
                        Date::from_calendar_date(year, Month::January, 1).ok()?,
                        time!(00:00:00),
                    )))
                }
                PeriodKind::Month => {
                    let month = ts.month().next();
                    let year = match month {
                        Month::January => ts.year() + 1,
                        _ => ts.year(),
                    };
                    Some(ts.replace_date_time(PrimitiveDateTime::new(
                        Date::from_calendar_date(year, month, 1).ok()?,
                        time!(00:00:00),
                    )))
                }
                PeriodKind::Duration(dur) => trunc(ts, dur).map(|ts| ts + dur),
            },
        };
        match current {
            Some(current) => match self.end {
                Bound::Unbounded => Some(current),
                Bound::Included(end) => match self.period {
                    PeriodKind::Year => {
                        if current.year() <= end.year() {
                            Some(current)
                        } else {
                            self.current = Bound::Unbounded;
                            None
                        }
                    }
                    PeriodKind::Month => {
                        if (current.year() < end.year())
                            || (current.year() == end.year()
                                && current.month() as u8 <= end.month() as u8)
                        {
                            Some(current)
                        } else {
                            self.current = Bound::Unbounded;
                            None
                        }
                    }
                    PeriodKind::Duration(dur) => {
                        if current < end + dur {
                            Some(current)
                        } else {
                            self.current = Bound::Unbounded;
                            None
                        }
                    }
                },
                Bound::Excluded(end) => {
                    if current < end {
                        Some(current)
                    } else {
                        self.current = Bound::Unbounded;
                        None
                    }
                }
            },
            None => {
                self.current = Bound::Unbounded;
                None
            }
        }
    }
}

impl Iterator for RangeIter {
    type Item = OffsetDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current_included()?;
        let next = match self.period {
            PeriodKind::Year => {
                let year = current.year() + 1;
                current.replace_date_time(PrimitiveDateTime::new(
                    Date::from_calendar_date(year, Month::January, 1).unwrap(),
                    time!(00:00:00),
                ))
            }
            PeriodKind::Month => {
                let month = current.month().next();
                let year = match month {
                    Month::January => current.year() + 1,
                    _ => current.year(),
                };
                current.replace_date_time(PrimitiveDateTime::new(
                    Date::from_calendar_date(year, month, 1).unwrap(),
                    time!(00:00:00),
                ))
            }
            PeriodKind::Duration(dur) => current + dur,
        };
        self.current = Bound::Included(next);
        Some(current)
    }
}

/// Useful extentions for [`Period`].
pub trait PeriodExt {
    /// Iterate ts in range, using this period as step.
    /// Note that if the range doesn't have a start point, it will return an empty iterator.
    ///
    /// # Example 1
    /// ```
    /// use indicator::Period;
    /// use exc::util::PeriodExt;
    /// use time::macros::{datetime, offset};
    ///
    /// let period = Period::weeks(offset!(+8), 1);
    /// let tss = period
    ///     .iterate(datetime!(2022-05-01 12:31:59 +08:00)..datetime!(2022-06-01 11:56:49 +08:00))
    ///     .collect::<Vec<_>>();
    /// assert_eq!(
    ///     vec![
    ///         datetime!(2022-04-25 00:00:00 +08:00),
    ///         datetime!(2022-05-02 00:00:00 +08:00),
    ///         datetime!(2022-05-09 00:00:00 +08:00),
    ///         datetime!(2022-05-16 00:00:00 +08:00),
    ///         datetime!(2022-05-23 00:00:00 +08:00),
    ///         datetime!(2022-05-30 00:00:00 +08:00),
    ///     ],
    ///     tss,
    /// );
    ///
    /// ```
    /// # Example 2
    /// ```
    /// use indicator::Period;
    /// use exc::util::PeriodExt;
    /// use time::macros::{datetime, offset};
    ///
    /// let period = Period::weeks(offset!(+8), 1);
    /// let tss = period
    ///     .iterate(datetime!(2022-05-01 12:31:59 +08:00)..=datetime!(2022-06-01 11:56:49 +08:00))
    ///     .collect::<Vec<_>>();
    /// assert_eq!(
    ///     vec![
    ///         datetime!(2022-04-25 00:00:00 +08:00),
    ///         datetime!(2022-05-02 00:00:00 +08:00),
    ///         datetime!(2022-05-09 00:00:00 +08:00),
    ///         datetime!(2022-05-16 00:00:00 +08:00),
    ///         datetime!(2022-05-23 00:00:00 +08:00),
    ///         datetime!(2022-05-30 00:00:00 +08:00),
    ///         datetime!(2022-06-06 00:00:00 +08:00),
    ///     ],
    ///     tss,
    /// );
    ///
    /// ```
    /// # Example 3
    /// ```
    /// use indicator::Period;
    /// use exc::util::PeriodExt;
    /// use time::macros::{datetime, offset};
    ///
    /// let period = Period::weeks(offset!(+8), 1);
    /// let tss = period
    ///     .iterate(datetime!(2022-04-25 00:00:00 +08:00)..=datetime!(2022-05-30 00:00:00 +08:00))
    ///     .collect::<Vec<_>>();
    /// assert_eq!(
    ///     vec![
    ///         datetime!(2022-04-25 00:00:00 +08:00),
    ///         datetime!(2022-05-02 00:00:00 +08:00),
    ///         datetime!(2022-05-09 00:00:00 +08:00),
    ///         datetime!(2022-05-16 00:00:00 +08:00),
    ///         datetime!(2022-05-23 00:00:00 +08:00),
    ///         datetime!(2022-05-30 00:00:00 +08:00),
    ///     ],
    ///     tss,
    /// );
    ///
    /// ```
    fn iterate<R: RangeBounds<OffsetDateTime>>(&self, range: R) -> RangeIter;
}

impl PeriodExt for Period {
    fn iterate<R: RangeBounds<OffsetDateTime>>(&self, range: R) -> RangeIter {
        let offset = self.utc_offset();
        let kind = self.kind();
        let current = match range.start_bound() {
            Bound::Excluded(ts) => Bound::Excluded((*ts).to_offset(offset)),
            Bound::Included(ts) => Bound::Included((*ts).to_offset(offset)),
            _ => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(end) => Bound::Included(*end),
            Bound::Excluded(end) => Bound::Excluded(*end),
        };
        RangeIter {
            period: kind,
            end,
            current,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use time::macros::{datetime, offset};

    #[test]
    fn test_year_included() {
        let period = Period::year(offset!(+8));
        let tss = period
            .iterate(datetime!(2021-04-25 00:00:00 +08:00)..=datetime!(2022-05-30 00:00:00 +08:00))
            .collect::<Vec<_>>();
        assert_eq!(
            vec![
                datetime!(2021-01-01 00:00:00 +08:00),
                datetime!(2022-01-01 00:00:00 +08:00),
            ],
            tss,
        );
    }

    #[test]
    fn test_year_excluded() {
        let period = Period::year(offset!(+8));
        let tss = period
            .iterate(datetime!(2021-04-25 00:00:00 +08:00)..datetime!(2022-01-01 00:00:00 +08:00))
            .collect::<Vec<_>>();
        assert_eq!(vec![datetime!(2021-01-01 00:00:00 +08:00),], tss,);
    }

    #[test]
    fn test_year_included_2() {
        let period = Period::year(offset!(+8));
        let tss = period
            .iterate(datetime!(2021-04-25 00:00:00 +08:00)..=datetime!(2022-01-01 00:00:00 +08:00))
            .collect::<Vec<_>>();
        assert_eq!(
            vec![
                datetime!(2021-01-01 00:00:00 +08:00),
                datetime!(2022-01-01 00:00:00 +08:00)
            ],
            tss,
        );
    }

    #[test]
    fn test_month_included() {
        let period = Period::month(offset!(+1));
        let tss = period
            .iterate(datetime!(2022-04-25 00:00:00 +01:00)..=datetime!(2022-05-01 00:00:00 +01:00))
            .collect::<Vec<_>>();
        assert_eq!(
            vec![
                datetime!(2022-04-01 00:00:00 +01:00),
                datetime!(2022-05-01 00:00:00 +01:00)
            ],
            tss,
        );
    }

    #[test]
    fn test_month_excluded() {
        let period = Period::month(offset!(+8));
        let tss = period
            .iterate(datetime!(2022-04-25 00:00:00 +08:00)..datetime!(2022-05-01 00:00:00 +08:00))
            .collect::<Vec<_>>();
        assert_eq!(vec![datetime!(2022-04-01 00:00:00 +08:00),], tss,);
    }
}
