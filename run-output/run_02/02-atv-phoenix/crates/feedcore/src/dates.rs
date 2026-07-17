//! Date parsing and normalization.
//!
//! All feed timestamps are normalized to UTC. Callers serialize with
//! [`to_rfc3339_z`] so stored values are RFC 3339 with a trailing `Z`.
//! Anything unparseable yields `None` — the caller stores `null` and never
//! substitutes the fetch time.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};

/// Serialize an instant as RFC 3339 in UTC with a `Z` suffix (seconds precision).
pub fn to_rfc3339_z(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Parse a date string, trying RFC 3339 first, then RFC 822.
pub fn parse_any(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    parse_rfc3339(s).or_else(|| parse_rfc822(s))
}

/// Parse an RFC 3339 / ISO 8601 instant with any numeric offset and optional
/// fractional seconds, normalized to UTC.
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn month_num(m: &str) -> Option<u32> {
    let m = m.to_ascii_lowercase();
    Some(match m.as_str() {
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
    })
}

/// Offset in seconds east of UTC for a named RFC 822 zone, per the pinned set.
fn named_zone_offset(z: &str) -> Option<i32> {
    let h = 3600;
    Some(match z {
        "GMT" | "UT" | "UTC" | "Z" => 0,
        "EST" => -5 * h,
        "EDT" => -4 * h,
        "CST" => -6 * h,
        "CDT" => -5 * h,
        "MST" => -7 * h,
        "MDT" => -6 * h,
        "PST" => -8 * h,
        "PDT" => -7 * h,
        _ => return None,
    })
}

/// Parse a numeric offset like `+0000`, `-0500`, `+00:00`.
fn numeric_offset(z: &str) -> Option<i32> {
    let bytes = z.as_bytes();
    if bytes.len() < 5 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let digits: String = z[1..].chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 4 {
        return None;
    }
    let hh: i32 = digits[0..2].parse().ok()?;
    let mm: i32 = digits[2..4].parse().ok()?;
    if hh > 23 || mm > 59 {
        return None;
    }
    Some(sign * (hh * 3600 + mm * 60))
}

/// Parse an RFC 822 / 1123 date (as used by RSS `pubDate`) to UTC.
///
/// Accepts 4-digit years, numeric offsets (`+0000`, `-0500`) and the zone
/// names `GMT UT Z EST EDT CST CDT MST MDT PST PDT`. An optional leading
/// day-of-week and optional seconds are tolerated.
pub fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    let mut s = s.trim();
    // Drop an optional "Wed, " style day-of-week prefix.
    if let Some(idx) = s.find(',') {
        if s[..idx].chars().all(|c| c.is_ascii_alphabetic()) {
            s = s[idx + 1..].trim_start();
        }
    }
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() < 5 {
        return None;
    }
    let day: u32 = tokens[0].parse().ok()?;
    let month = month_num(tokens[1])?;
    let year: i32 = match tokens[2].len() {
        4 => tokens[2].parse().ok()?,
        2 => {
            let y: i32 = tokens[2].parse().ok()?;
            if y < 70 {
                2000 + y
            } else {
                1900 + y
            }
        }
        _ => return None,
    };

    let time_parts: Vec<&str> = tokens[3].split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = if time_parts.len() >= 3 {
        time_parts[2].parse().ok()?
    } else {
        0
    };

    let zone = tokens[4];
    let offset_secs =
        numeric_offset(zone).or_else(|| named_zone_offset(&zone.to_ascii_uppercase()))?;

    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;
    // The naive wall-clock is in the given zone; subtract the offset for UTC.
    let utc = Utc.from_utc_datetime(&naive) - chrono::Duration::seconds(offset_secs as i64);
    Some(utc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc822_gmt() {
        let dt = parse_rfc822("Wed, 02 Oct 2002 13:00:00 GMT").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2002-10-02T13:00:00Z");
    }

    #[test]
    fn rfc822_numeric_offset() {
        let dt = parse_rfc822("Mon, 15 Jul 2024 10:30:00 -0500").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2024-07-15T15:30:00Z");
    }

    #[test]
    fn rfc822_est_zone() {
        let dt = parse_rfc822("Mon, 15 Jul 2024 10:30:00 EST").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2024-07-15T15:30:00Z");
    }

    #[test]
    fn rfc822_no_dow_no_seconds() {
        let dt = parse_rfc822("15 Jul 2024 10:30 PDT").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2024-07-15T17:30:00Z");
    }

    #[test]
    fn rfc3339_fractional_and_offset() {
        let dt = parse_rfc3339("2024-07-15T10:30:00.500-05:00").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2024-07-15T15:30:00Z");
    }

    #[test]
    fn rfc3339_z() {
        let dt = parse_rfc3339("2024-01-01T00:00:00Z").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2024-01-01T00:00:00Z");
    }

    #[test]
    fn unparseable_is_none() {
        assert!(parse_any("not a date").is_none());
        assert!(parse_any("").is_none());
    }

    #[test]
    fn parse_any_prefers_valid() {
        assert!(parse_any("2024-07-15T10:30:00Z").is_some());
        assert!(parse_any("Mon, 15 Jul 2024 10:30:00 GMT").is_some());
    }
}
