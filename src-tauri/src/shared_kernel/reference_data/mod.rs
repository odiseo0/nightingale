//! Shared country and time zone names. Source: docs/technical/reference-data.md.

pub fn is_country_code(value: &str) -> bool {
    include_str!("countries.txt")
        .lines()
        .any(|code| code == value)
}

pub fn is_time_zone(value: &str) -> bool {
    include_str!("time_zones.txt")
        .lines()
        .any(|zone| zone == value)
}
