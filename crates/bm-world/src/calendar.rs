//! Calendar dates for the world: schedules, ages, service time.
//!
//! Proleptic Gregorian calendar; day numbers count from 1970-01-01 (the conversion is
//! Howard Hinnant's `days_from_civil` / `civil_from_days`).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A day, stored as the number of days since 1970-01-01.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Date {
    /// The date `year-month-day`. Panics on an invalid date.
    pub fn new(year: i32, month: u32, day: u32) -> Date {
        assert!((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month), "invalid date {year}-{month}-{day}");
        let y = if month <= 2 { year - 1 } else { year };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (month + 9) % 12;
        let doy = (153 * mp + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy as i32;
        Date(era * 146_097 + doe - 719_468)
    }

    /// Year, month (1-12) and day of the month.
    pub fn ymd(self) -> (i32, u32, u32) {
        let z = self.0 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        (yoe + era * 400 + i32::from(month <= 2), month, day)
    }

    pub fn year(self) -> i32 {
        self.ymd().0
    }

    pub fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday.
        [Weekday::Thursday, Weekday::Friday, Weekday::Saturday, Weekday::Sunday, Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday]
            [self.0.rem_euclid(7) as usize]
    }

    pub fn add_days(self, days: i32) -> Date {
        Date(self.0 + days)
    }

    /// Days from `earlier` to `self`.
    pub fn days_since(self, earlier: Date) -> i32 {
        self.0 - earlier.0
    }

    /// Completed years of age on `on` for someone born on `self`.
    pub fn age_on(self, on: Date) -> i32 {
        let (by, bm, bd) = self.ymd();
        let (y, m, d) = on.ymd();
        y - by - i32::from((m, d) < (bm, bd))
    }

    /// Parse `YYYY-MM-DD`.
    pub fn parse(s: &str) -> Option<Date> {
        let mut it = s.split('-');
        let (y, m, d) = (it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?);
        if it.next().is_some() || !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
            return None;
        }
        Some(Date::new(y, m, d))
    }
}

pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, m, d) = self.ymd();
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Date, D::Error> {
        let s = String::deserialize(d)?;
        Date::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid date '{s}'")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip_and_know_their_weekday() {
        assert_eq!(Date::new(1970, 1, 1), Date(0));
        assert_eq!(Date::new(2000, 3, 1).days_since(Date::new(2000, 2, 28)), 2);
        assert_eq!(Date::new(1900, 3, 1).days_since(Date::new(1900, 2, 28)), 1);
        // Opening Day 2027 is a Thursday, the first day of KBO spring camp a Monday.
        assert_eq!(Date::new(2027, 3, 25).weekday(), Weekday::Thursday);
        assert_eq!(Date::new(2027, 1, 25).weekday(), Weekday::Monday);
        let mut d = Date::new(1899, 12, 25);
        for _ in 0..60_000 {
            let (y, m, day) = d.ymd();
            assert_eq!(Date::new(y, m, day), d);
            d = d.add_days(1);
        }
        assert_eq!(Date::parse("2027-02-29"), None);
        assert_eq!(Date::parse("2028-02-29"), Some(Date::new(2028, 2, 29)));
        assert_eq!(Date::new(2027, 2, 1).to_string(), "2027-02-01");
    }

    #[test]
    fn ages_count_completed_years() {
        let born = Date::new(2004, 7, 15);
        assert_eq!(born.age_on(Date::new(2027, 7, 14)), 22);
        assert_eq!(born.age_on(Date::new(2027, 7, 15)), 23);
        let json = serde_json::to_string(&born).unwrap();
        assert_eq!(json, "\"2004-07-15\"");
        assert_eq!(serde_json::from_str::<Date>(&json).unwrap(), born);
    }
}
