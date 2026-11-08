use super::availability::{Interval, available_intervals, fitting, merge, subtract};
use super::{
    application::{BlockState, ExceptionState, RuleState},
    domain::{ExceptionKind, ReasonCode, Status},
};
use jiff::{Timestamp, tz::TimeZone};
use std::str::FromStr;

fn ts(value: &str) -> Timestamp {
    Timestamp::from_str(value).unwrap()
}

fn rule(weekday: i64, start: i64, end: i64, valid_from: &str, valid_until: Option<&str>) -> RuleState {
    RuleState {
        rule_id: "019a1234-0000-7000-8000-0000000000a1".into(),
        professional_id: "019a1234-0000-7000-8000-000000000008".into(),
        iso_weekday: weekday,
        local_start_minute: start,
        local_end_minute: end,
        valid_from: valid_from.into(),
        valid_until: valid_until.map(str::to_owned),
        status: Status::Active.as_str().into(),
    }
}

fn closed(date: &str) -> ExceptionState {
    ExceptionState {
        exception_id: "019a1234-0000-7000-8000-0000000000a2".into(),
        professional_id: "019a1234-0000-7000-8000-000000000008".into(),
        local_date: date.into(),
        kind: ExceptionKind::Closed.as_str().into(),
        local_start_minute: None,
        local_end_minute: None,
        status: Status::Active.as_str().into(),
    }
}

fn replacement(date: &str, start: i64, end: i64) -> ExceptionState {
    ExceptionState {
        kind: ExceptionKind::Replacement.as_str().into(),
        local_start_minute: Some(start),
        local_end_minute: Some(end),
        ..closed(date)
    }
}

fn block(starts: &str, ends: &str) -> BlockState {
    BlockState {
        block_id: "019a1234-0000-7000-8000-0000000000a3".into(),
        professional_id: "019a1234-0000-7000-8000-000000000008".into(),
        block_account_id: None,
        starts_at: starts.into(),
        ends_at: ends.into(),
        reason_code: Some(ReasonCode::Administrative.as_str().into()),
        status: Status::Active.as_str().into(),
    }
}

fn interval(starts: &str, ends: &str) -> Interval {
    Interval::new(ts(starts), ts(ends)).unwrap()
}

/// Caracas es UTC-4 sin DST: 08:00 local = 12:00Z.
fn caracas() -> TimeZone {
    TimeZone::get("America/Caracas").unwrap()
}

#[test]
fn merge_unions_overlaps_and_contiguous_without_duplicates() {
    let merged = merge(vec![
        interval("2026-05-06T14:00:00Z", "2026-05-06T15:00:00Z"),
        interval("2026-05-06T12:00:00Z", "2026-05-06T13:00:00Z"),
        interval("2026-05-06T13:00:00Z", "2026-05-06T13:30:00Z"),
        interval("2026-05-06T12:30:00Z", "2026-05-06T12:45:00Z"),
    ]);
    assert_eq!(
        merged,
        [
            interval("2026-05-06T12:00:00Z", "2026-05-06T13:30:00Z"),
            interval("2026-05-06T14:00:00Z", "2026-05-06T15:00:00Z"),
        ]
    );
}

#[test]
fn subtract_handles_overlap_contiguity_and_gaps() {
    // Contiguous cuts leave no gap and no overlap.
    let base = merge(vec![interval(
        "2026-05-06T12:00:00Z",
        "2026-05-06T18:00:00Z",
    )]);
    let result = subtract(
        base,
        &[
            interval("2026-05-06T13:00:00Z", "2026-05-06T14:00:00Z"),
            interval("2026-05-06T14:00:00Z", "2026-05-06T15:00:00Z"),
            interval("2026-05-06T16:00:00Z", "2026-05-06T16:30:00Z"),
        ],
    );
    assert_eq!(
        result,
        [
            interval("2026-05-06T12:00:00Z", "2026-05-06T13:00:00Z"),
            interval("2026-05-06T15:00:00Z", "2026-05-06T16:00:00Z"),
            interval("2026-05-06T16:30:00Z", "2026-05-06T18:00:00Z"),
        ]
    );
}

#[test]
fn fitting_keeps_only_intervals_containing_the_duration() {
    let intervals = vec![
        interval("2026-05-06T12:00:00Z", "2026-05-06T13:00:00Z"),
        interval("2026-05-06T14:00:00Z", "2026-05-06T16:00:00Z"),
    ];
    assert_eq!(fitting(intervals.clone(), 60).len(), 2);
    assert_eq!(fitting(intervals.clone(), 61).len(), 1);
    assert!(fitting(intervals, 240).is_empty());
}

#[test]
fn rules_union_exceptions_precedence_and_duration_filter() {
    // Dos reglas solapadas unen 08:00-14:00; duración 360 min conserva, 361 excluye.
    let rules = [
        rule(3, 480, 720, "2026-01-01", None),
        rule(3, 600, 840, "2026-01-01", None),
    ];
    let result = available_intervals(
        &caracas(),
        &rules,
        &[],
        &[],
        &[],
        ts("2026-05-06T04:00:00Z"),
        ts("2026-05-07T04:00:00Z"),
        360,
    )
    .unwrap();
    assert_eq!(result, [interval("2026-05-06T12:00:00Z", "2026-05-06T18:00:00Z")]);

    let result = available_intervals(
        &caracas(),
        &rules,
        &[],
        &[],
        &[],
        ts("2026-05-06T04:00:00Z"),
        ts("2026-05-07T04:00:00Z"),
        361,
    )
    .unwrap();
    assert!(result.is_empty());

    // closed vacía el día aunque haya reglas.
    let result = available_intervals(
        &caracas(),
        &rules,
        &[closed("2026-05-06")],
        &[],
        &[],
        ts("2026-05-06T04:00:00Z"),
        ts("2026-05-07T04:00:00Z"),
        30,
    )
    .unwrap();
    assert!(result.is_empty());

    // replacement sustituye el horario habitual.
    let result = available_intervals(
        &caracas(),
        &rules,
        &[replacement("2026-05-06", 540, 600)],
        &[],
        &[],
        ts("2026-05-06T04:00:00Z"),
        ts("2026-05-07T04:00:00Z"),
        30,
    )
    .unwrap();
    assert_eq!(result, [interval("2026-05-06T13:00:00Z", "2026-05-06T14:00:00Z")]);
}

#[test]
fn blocks_and_occupancy_subtract_with_half_open_edges() {
    // Regla 08:00-14:00; cita contigua 09:00-10:00 y 10:00-11:00; bloqueo 12:00-13:00.
    let rules = [rule(3, 480, 840, "2026-01-01", None)];
    let occupancy = [
        interval("2026-05-06T13:00:00Z", "2026-05-06T14:00:00Z"),
        interval("2026-05-06T14:00:00Z", "2026-05-06T15:00:00Z"),
    ];
    let blocks = [block("2026-05-06T16:00:00Z", "2026-05-06T17:00:00Z")];
    let result = available_intervals(
        &caracas(),
        &rules,
        &[],
        &blocks,
        &occupancy,
        ts("2026-05-06T04:00:00Z"),
        ts("2026-05-07T04:00:00Z"),
        15,
    )
    .unwrap();
    // Libre 08:00-09:00, 11:00-12:00 (citas contiguas no se solapan) y 13:00-14:00 (tras el bloqueo).
    assert_eq!(
        result,
        [
            interval("2026-05-06T12:00:00Z", "2026-05-06T13:00:00Z"),
            interval("2026-05-06T15:00:00Z", "2026-05-06T16:00:00Z"),
            interval("2026-05-06T17:00:00Z", "2026-05-06T18:00:00Z"),
        ]
    );
}

#[test]
fn validity_windows_and_multi_day_ranges() {
    // Vigencia inclusiva: regla solo el 2026-05-05..2026-05-07.
    let rules = [rule(2, 480, 600, "2026-05-05", Some("2026-05-06"))];
    let result = available_intervals(
        &caracas(),
        &rules,
        &[],
        &[],
        &[],
        ts("2026-05-05T04:00:00Z"),
        ts("2026-05-08T04:00:00Z"),
        30,
    )
    .unwrap();
    // 05-05 es martes (dentro), 06-05 miércoles (fuera por vigencia y día), 07-05 jueves.
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start, ts("2026-05-05T12:00:00Z"));
}

#[test]
fn dst_gap_and_fold_are_rejected_not_resolved() {
    let new_york = TimeZone::get("America/New_York").unwrap();
    // 2026-03-08 02:00-03:00 no existe (gap).
    let rules = [rule(7, 120, 180, "2026-01-01", None)];
    let result = available_intervals(
        &new_york,
        &rules,
        &[],
        &[],
        &[],
        ts("2026-03-08T05:00:00Z"),
        ts("2026-03-09T04:00:00Z"),
        15,
    );
    assert!(result.is_err());

    // 2026-11-01 01:00-02:00 es ambiguo (fold).
    let rules = [rule(7, 60, 120, "2026-01-01", None)];
    let result = available_intervals(
        &new_york,
        &rules,
        &[],
        &[],
        &[],
        ts("2026-11-01T04:00:00Z"),
        ts("2026-11-02T05:00:00Z"),
        15,
    );
    assert!(result.is_err());
}

#[test]
fn end_1440_lands_on_next_day_and_day_length_follows_dst() {
    // Regla 00:00-1440 en Caracas (día de 24h, sin DST): 04:00Z - 28:00Z (= día siguiente 04:00Z).
    let rules = [rule(7, 0, 1440, "2026-01-01", None)];
    let result = available_intervals(
        &caracas(),
        &rules,
        &[],
        &[],
        &[],
        ts("2026-05-03T04:00:00Z"),
        ts("2026-05-04T04:00:00Z"),
        1440,
    )
    .unwrap();
    assert_eq!(result, [interval("2026-05-03T04:00:00Z", "2026-05-04T04:00:00Z")]);

    // En Nueva York el domingo del cambio (gap de 1h) el día mide 23h: 1440 no cabe.
    let new_york = TimeZone::get("America/New_York").unwrap();
    let result = available_intervals(
        &new_york,
        &rules,
        &[],
        &[],
        &[],
        ts("2026-03-08T05:00:00Z"),
        ts("2026-03-09T04:00:00Z"),
        1440,
    );
    assert!(result.is_err() || result.unwrap().is_empty());
}

#[test]
fn invalid_range_and_duration_are_rejected() {
    let rules = [rule(3, 480, 720, "2026-01-01", None)];
    assert!(
        available_intervals(&caracas(), &rules, &[], &[], &[], ts("2026-05-07T04:00:00Z"), ts("2026-05-06T04:00:00Z"), 30)
            .is_err()
    );
    assert!(
        available_intervals(&caracas(), &rules, &[], &[], &[], ts("2026-05-06T04:00:00Z"), ts("2026-05-07T04:00:00Z"), 0)
            .is_err()
    );
    assert!(
        available_intervals(&caracas(), &rules, &[], &[], &[], ts("2026-05-06T04:00:00Z"), ts("2026-05-07T04:00:00Z"), 1441)
            .is_err()
    );
}
