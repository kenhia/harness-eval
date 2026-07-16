//! Date parsing and the pinned storage format.
//!
//! Every instant feedhub stores is normalized to UTC and serialized as RFC 3339
//! with a `Z` suffix and second precision, e.g. `2024-03-01T12:30:00Z`. That
//! format is fixed-width, so lexicographic ordering of the stored strings is
//! also chronological ordering — which is what lets SQLite sort and range-filter
//! `published_at` directly.

use chrono::{DateTime, FixedOffset, NaiveDate, SecondsFormat, TimeDelta, TimeZone, Timelike, Utc};

/// Serialize an instant in the pinned storage format: RFC 3339, UTC, `Z`.
///
/// Sub-second precision is dropped, so this rounds *down* to the second.
pub fn format_utc(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Serialize an instant in the storage format, rounding *up* to the next whole
/// second when it has a fractional part.
///
/// This is what a query bound needs. Stored instants are whole seconds, so
/// comparing them against a bound with fractional seconds has to round the bound
/// in the direction that preserves the comparison:
///
/// * `published_at >= since` — an entry at 12:00:00 is not at or after
///   12:00:00.5, so `since` must round up to 12:00:01 to exclude it.
/// * `published_at < until` — an entry at 12:00:00 *is* before 12:00:00.5, so
///   `until` must round up to 12:00:01 to keep it.
///
/// Rounding down would get the second case backwards and silently drop entries
/// from inside the requested window.
pub fn format_utc_ceil(dt: DateTime<Utc>) -> String {
    let rounded = match dt.nanosecond() {
        0 => dt,
        // Leap seconds report nanosecond() >= 1_000_000_000; truncating first
        // keeps the arithmetic on the second boundary either way.
        _ => dt
            .with_nanosecond(0)
            .unwrap_or(dt)
            .checked_add_signed(TimeDelta::seconds(1))
            .unwrap_or(dt),
    };
    format_utc(rounded)
}

/// Parse an RFC 3339 timestamp with any numeric offset and optional fractional
/// seconds, returning it as UTC. Returns `None` if the input is not RFC 3339.
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Zone names feedhub accepts in RFC 822 dates, with their offsets in hours.
const ZONE_NAMES: &[(&str, i32)] = &[
    ("GMT", 0),
    ("UT", 0),
    ("Z", 0),
    ("EST", -5),
    ("EDT", -4),
    ("CST", -6),
    ("CDT", -5),
    ("MST", -7),
    ("MDT", -6),
    ("PST", -8),
    ("PDT", -7),
];

const MONTHS: &[&str] = &[
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

/// Parse an RFC 822 / RFC 1123 date (the RSS `pubDate` format), returning it as
/// UTC. Returns `None` for anything that does not parse — callers store that as
/// a null `published_at` rather than substituting another instant.
///
/// Accepts an optional leading day-of-week, 2- or 4-digit years, an optional
/// seconds field, numeric offsets like `+0000` / `-0500`, and the zone names in
/// [`ZONE_NAMES`]. A date with an unrecognized or missing zone does not parse.
pub fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    // Drop the optional "Ddd, " day-of-week prefix; it carries no information we
    // need and feeds disagree about whether it is even present.
    let rest = match s.find(',') {
        Some(i) => &s[i + 1..],
        None => s,
    };

    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 5 {
        return None;
    }

    let day: u32 = tokens[0].parse().ok()?;
    let month = month_from_name(tokens[1])?;
    let year = parse_year(tokens[2])?;
    let (hour, minute, second) = parse_time(tokens[3])?;
    let offset_secs = parse_zone(tokens[4])?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let naive = date.and_hms_opt(hour, minute, second)?;
    let offset = FixedOffset::east_opt(offset_secs)?;
    offset
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

fn month_from_name(name: &str) -> Option<u32> {
    let lower = name.to_ascii_lowercase();
    MONTHS
        .iter()
        .position(|m| *m == lower)
        .map(|i| i as u32 + 1)
}

/// RFC 2822 §4.3 year handling: 4 digits are literal, 2-digit years 00-49 mean
/// 2000-2049 and 50-99 mean 1950-1999, 3-digit years are offsets from 1900.
fn parse_year(token: &str) -> Option<i32> {
    if !token.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let value: i32 = token.parse().ok()?;
    match token.len() {
        4.. => Some(value),
        3 => Some(1900 + value),
        2 => Some(if value < 50 {
            2000 + value
        } else {
            1900 + value
        }),
        _ => None,
    }
}

fn parse_time(token: &str) -> Option<(u32, u32, u32)> {
    let mut parts = token.split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = parts.next()?.parse().ok()?;
    let second: u32 = match parts.next() {
        Some(s) => s.parse().ok()?,
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((hour, minute, second))
}

/// Parse a numeric offset (`+0000`, `-0500`) or one of [`ZONE_NAMES`], as
/// seconds east of UTC.
fn parse_zone(token: &str) -> Option<i32> {
    if let Some(sign) = match token.as_bytes().first()? {
        b'+' => Some(1),
        b'-' => Some(-1),
        _ => None,
    } {
        let digits = &token[1..];
        if digits.len() != 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let hours: i32 = digits[..2].parse().ok()?;
        let minutes: i32 = digits[2..].parse().ok()?;
        if minutes > 59 {
            return None;
        }
        return Some(sign * (hours * 3600 + minutes * 60));
    }

    let upper = token.to_ascii_uppercase();
    ZONE_NAMES
        .iter()
        .find(|(name, _)| *name == upper)
        .map(|(_, hours)| hours * 3600)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        parse_rfc3339(s).expect("test literal should be RFC 3339")
    }

    #[test]
    fn rfc822_numeric_offsets() {
        assert_eq!(
            parse_rfc822("Fri, 01 Mar 2024 12:30:00 +0000"),
            Some(utc("2024-03-01T12:30:00Z"))
        );
        assert_eq!(
            parse_rfc822("Fri, 01 Mar 2024 12:30:00 -0500"),
            Some(utc("2024-03-01T17:30:00Z"))
        );
        assert_eq!(
            parse_rfc822("1 Mar 2024 12:30:00 +0530"),
            Some(utc("2024-03-01T07:00:00Z"))
        );
    }

    #[test]
    fn rfc822_zone_names() {
        let cases = [
            ("GMT", "2024-03-01T12:00:00Z"),
            ("UT", "2024-03-01T12:00:00Z"),
            ("Z", "2024-03-01T12:00:00Z"),
            ("EST", "2024-03-01T17:00:00Z"),
            ("EDT", "2024-03-01T16:00:00Z"),
            ("CST", "2024-03-01T18:00:00Z"),
            ("CDT", "2024-03-01T17:00:00Z"),
            ("MST", "2024-03-01T19:00:00Z"),
            ("MDT", "2024-03-01T18:00:00Z"),
            ("PST", "2024-03-01T20:00:00Z"),
            ("PDT", "2024-03-01T19:00:00Z"),
        ];
        for (zone, expected) in cases {
            let input = format!("Fri, 01 Mar 2024 12:00:00 {zone}");
            assert_eq!(parse_rfc822(&input), Some(utc(expected)), "zone {zone}");
        }
    }

    #[test]
    fn rfc822_optional_pieces() {
        // No day-of-week, no seconds.
        assert_eq!(
            parse_rfc822("01 Mar 2024 12:30 GMT"),
            Some(utc("2024-03-01T12:30:00Z"))
        );
        // Two-digit years.
        assert_eq!(
            parse_rfc822("Fri, 01 Mar 24 12:30:00 GMT"),
            Some(utc("2024-03-01T12:30:00Z"))
        );
        assert_eq!(
            parse_rfc822("Wed, 01 Mar 95 12:30:00 GMT"),
            Some(utc("1995-03-01T12:30:00Z"))
        );
    }

    #[test]
    fn rfc822_rejects_garbage() {
        for input in [
            "",
            "not a date",
            "Fri, 01 Mar 2024 12:30:00",       // no zone
            "Fri, 01 Mar 2024 12:30:00 CEST",  // unlisted zone name
            "Fri, 32 Mar 2024 12:30:00 GMT",   // impossible day
            "Fri, 01 Foo 2024 12:30:00 GMT",   // bad month
            "Fri, 01 Mar 2024 25:30:00 GMT",   // impossible hour
            "Fri, 01 Mar 2024 12:30:00 +5:00", // malformed offset
        ] {
            assert_eq!(parse_rfc822(input), None, "expected None for {input:?}");
        }
    }

    #[test]
    fn rfc3339_offsets_and_fractions() {
        assert_eq!(
            parse_rfc3339("2024-03-01T12:30:00Z"),
            Some(utc("2024-03-01T12:30:00Z"))
        );
        // Sub-second precision survives parsing; it is dropped by `format_utc`
        // when the instant is written to the database.
        assert_eq!(
            parse_rfc3339("2024-03-01T12:30:00.123456+05:30").map(format_utc),
            Some("2024-03-01T07:00:00Z".to_string())
        );
        assert_eq!(
            parse_rfc3339("2024-03-01T12:30:00-08:00"),
            Some(utc("2024-03-01T20:30:00Z"))
        );
        assert_eq!(parse_rfc3339("2024-03-01"), None);
        assert_eq!(parse_rfc3339("yesterday"), None);
    }

    #[test]
    fn ceiling_rounds_up_only_when_there_is_a_fraction() {
        // A whole second is already the bound it means.
        assert_eq!(
            format_utc_ceil(utc("2024-03-01T12:00:00Z")),
            "2024-03-01T12:00:00Z"
        );
        // Anything past it belongs to the next whole second.
        assert_eq!(
            format_utc_ceil(utc("2024-03-01T12:00:00.001Z")),
            "2024-03-01T12:00:01Z"
        );
        assert_eq!(
            format_utc_ceil(utc("2024-03-01T12:00:00.999999Z")),
            "2024-03-01T12:00:01Z"
        );
        // Rounding up across a minute boundary carries.
        assert_eq!(
            format_utc_ceil(utc("2024-03-01T12:00:59.5Z")),
            "2024-03-01T12:01:00Z"
        );
    }

    #[test]
    fn storage_format_is_fixed_width_utc() {
        let dt = utc("2024-03-01T12:30:00.987+02:00");
        assert_eq!(format_utc(dt), "2024-03-01T10:30:00Z");
    }
}
