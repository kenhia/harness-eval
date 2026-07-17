//! Date parsing for feed formats.
//!
//! Two entry points are combined in [`parse_date`]: RFC 3339 (Atom) and
//! RFC 822/1123 (RSS). Every successfully parsed value is normalized to UTC.

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

/// Parse a date string from a feed, trying RFC 3339 then RFC 822.
///
/// Returns `None` when the value is missing or cannot be parsed; callers must
/// never substitute the fetch time for a `None` result.
pub fn parse_date(raw: &str) -> Option<DateTime<Utc>> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    parse_rfc3339(s).or_else(|| parse_rfc822(s))
}

/// RFC 3339: any numeric offset (or `Z`) and optional fractional seconds.
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Named time zones accepted in RFC 822 dates, mapped to offset seconds.
fn named_zone_offset(name: &str) -> Option<i32> {
    let secs = match name {
        "GMT" | "UT" | "UTC" | "Z" => 0,
        "EST" => -5 * 3600,
        "EDT" => -4 * 3600,
        "CST" => -6 * 3600,
        "CDT" => -5 * 3600,
        "MST" => -7 * 3600,
        "MDT" => -6 * 3600,
        "PST" => -8 * 3600,
        "PDT" => -7 * 3600,
        _ => return None,
    };
    Some(secs)
}

fn month_number(name: &str) -> Option<u32> {
    let m = match &name.to_ascii_lowercase()[..name.len().min(3)] {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    Some(m)
}

fn zone_offset(tok: &str) -> Option<FixedOffset> {
    if let Some(secs) = named_zone_offset(tok) {
        return FixedOffset::east_opt(secs);
    }
    // Numeric: +HHMM / -HHMM (optionally with a colon).
    let bytes = tok.as_bytes();
    if bytes.len() >= 5 && (bytes[0] == b'+' || bytes[0] == b'-') {
        let sign = if bytes[0] == b'-' { -1 } else { 1 };
        let digits: String = tok[1..].chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 4 {
            let hh: i32 = digits[0..2].parse().ok()?;
            let mm: i32 = digits[2..4].parse().ok()?;
            return FixedOffset::east_opt(sign * (hh * 3600 + mm * 60));
        }
    }
    None
}

/// RFC 822/1123: `[Wkd,] DD Mon YYYY HH:MM[:SS] ZONE`.
pub fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    // Drop an optional leading weekday token (`Mon,` / `Mon`).
    let rest = match s.split_once(',') {
        Some((_, r)) => r.trim(),
        None => s.trim(),
    };
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 5 {
        return None;
    }
    let day: u32 = tokens[0].parse().ok()?;
    let month = month_number(tokens[1])?;
    let year: i32 = tokens[2].parse().ok()?;

    let time: Vec<&str> = tokens[3].split(':').collect();
    if time.len() < 2 {
        return None;
    }
    let hour: u32 = time[0].parse().ok()?;
    let minute: u32 = time[1].parse().ok()?;
    let second: u32 = if time.len() >= 3 {
        time[2].parse().ok()?
    } else {
        0
    };

    let offset = zone_offset(tokens[4])?;
    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;
    let dt = offset.from_local_datetime(&naive).single()?;
    Some(dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn rfc3339_offset_and_fractional() {
        assert_eq!(
            parse_date("2021-03-04T05:06:07-05:00"),
            Some(utc("2021-03-04T10:06:07Z"))
        );
        assert_eq!(
            parse_date("2021-03-04T05:06:07.250Z"),
            Some(utc("2021-03-04T05:06:07.250Z"))
        );
    }

    #[test]
    fn rfc822_numeric_offsets() {
        assert_eq!(
            parse_date("Thu, 04 Mar 2021 05:06:07 +0000"),
            Some(utc("2021-03-04T05:06:07Z"))
        );
        assert_eq!(
            parse_date("04 Mar 2021 05:06:07 -0500"),
            Some(utc("2021-03-04T10:06:07Z"))
        );
    }

    #[test]
    fn rfc822_named_zones() {
        assert_eq!(
            parse_date("Thu, 04 Mar 2021 00:00:00 GMT"),
            Some(utc("2021-03-04T00:00:00Z"))
        );
        assert_eq!(
            parse_date("Thu, 04 Mar 2021 00:00:00 EST"),
            Some(utc("2021-03-04T05:00:00Z"))
        );
        assert_eq!(
            parse_date("Thu, 04 Mar 2021 00:00:00 PDT"),
            Some(utc("2021-03-04T07:00:00Z"))
        );
        assert_eq!(
            parse_date("Thu, 04 Mar 2021 00:00:00 Z"),
            Some(utc("2021-03-04T00:00:00Z"))
        );
    }

    #[test]
    fn rfc822_no_seconds() {
        assert_eq!(
            parse_date("04 Mar 2021 05:06 UT"),
            Some(utc("2021-03-04T05:06:00Z"))
        );
    }

    #[test]
    fn unparseable_is_none() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("not a date"), None);
        assert_eq!(parse_date("2021-13-40T00:00:00Z"), None);
    }
}
