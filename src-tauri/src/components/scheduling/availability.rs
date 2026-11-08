//! Pure availability interval engine: no storage, no HTTP, no framework state.
//! All intervals are half-open `[start,end)` UTC instants. Civil times convert
//! strictly (ambiguous or nonexistent local times are rejected, never resolved).
use super::application::{BlockState, ExceptionState, RuleState};
use super::domain::{SchedulingError, civil_interval_to_utc};
use jiff::{Span, Timestamp, civil::Date, tz::TimeZone};
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, SchedulingError>;

/// A half-open UTC interval `[start,end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    pub start: Timestamp,
    pub end: Timestamp,
}

impl Interval {
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self> {
        if start >= end {
            return Err(SchedulingError::validation(
                "el intervalo exige inicio menor que fin",
            ));
        }

        Ok(Self { start, end })
    }

    pub fn minutes(&self) -> i64 {
        (self.end.as_second() - self.start.as_second()) / 60
    }
}

/// Merges overlapping or contiguous intervals into ordered, disjoint pieces.
pub fn merge(mut intervals: Vec<Interval>) -> Vec<Interval> {
    intervals.sort_by_key(|interval| (interval.start, interval.end));
    let mut merged: Vec<Interval> = Vec::with_capacity(intervals.len());

    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            // Contiguity (end == next start) unions; half-open keeps it disjoint.
            if interval.start <= last.end {
                if interval.end > last.end {
                    last.end = interval.end;
                }

                continue;
            }
        }

        merged.push(interval);
    }

    merged
}

/// Subtracts `cuts` (merged or not) from `base`, preserving order and the
/// half-open semantics; contiguous cuts leave no gap and no overlap.
pub fn subtract(base: Vec<Interval>, cuts: &[Interval]) -> Vec<Interval> {
    let cuts = merge(cuts.to_vec());
    let mut result = Vec::new();

    for mut interval in base {
        for cut in &cuts {
            if cut.end <= interval.start || cut.start >= interval.end {
                continue;
            }

            if cut.start > interval.start {
                result.push(Interval {
                    start: interval.start,
                    end: cut.start,
                });
            }

            if cut.end < interval.end {
                interval.start = cut.end;
            } else {
                interval.start = interval.end;
                break;
            }
        }

        if interval.start < interval.end {
            result.push(interval);
        }
    }

    result
}

/// Keeps intervals long enough to contain `duration_minutes` wholly.
pub fn fitting(intervals: Vec<Interval>, duration_minutes: i64) -> Vec<Interval> {
    intervals
        .into_iter()
        .filter(|interval| interval.minutes() >= duration_minutes)
        .collect()
}

struct DailyMinutes {
    start: i64,
    end: i64,
}

/// The effective open minutes for one civil date: an active exception replaces
/// the whole day (`closed` leaves none); otherwise the union of active rules
/// covering that date applies.
fn daily_minutes(
    date: Date,
    rules: &[RuleState],
    exceptions: &[ExceptionState],
) -> Vec<DailyMinutes> {
    let date_text = date.to_string();

    if let Some(exception) = exceptions
        .iter()
        .find(|exception| exception.status == "active" && exception.local_date == date_text)
    {
        return match (exception.local_start_minute, exception.local_end_minute) {
            (Some(start), Some(end)) => vec![DailyMinutes { start, end }],
            _ => vec![],
        };
    }

    let weekday = i64::from(date.weekday().to_monday_one_offset());
    let mut minutes: Vec<DailyMinutes> = rules
        .iter()
        .filter(|rule| {
            rule.status == "active"
                && rule.iso_weekday == weekday
                && rule.valid_from.as_str() <= date_text.as_str()
                && rule
                    .valid_until
                    .as_deref()
                    .is_none_or(|until| date_text.as_str() <= until)
        })
        .map(|rule| DailyMinutes {
            start: rule.local_start_minute,
            end: rule.local_end_minute,
        })
        .collect();

    // Union of overlapping rules before converting to civil intervals.
    minutes.sort_by_key(|m| (m.start, m.end));
    let mut merged: Vec<DailyMinutes> = Vec::with_capacity(minutes.len());

    for span in minutes {
        if let Some(last) = merged.last_mut()
            && span.start <= last.end
        {
            if span.end > last.end {
                last.end = span.end;
            }
            continue;
        }

        merged.push(span);
    }

    merged
}

/// Evaluates availability for the civil dates touched by `[from, until)`:
/// rules and exceptions define the open minutes per date, converted strictly
/// to UTC and clipped to the query range; blocks and occupancy are subtracted;
/// results are merged, ordered and only intervals able to contain
/// `duration_minutes` (1..=1440) survive.
/// Agenda inputs for one evaluation, grouped to keep the engine signature small.
pub struct AgendaInput<'a> {
    pub rules: &'a [RuleState],
    pub exceptions: &'a [ExceptionState],
    pub blocks: &'a [BlockState],
}

pub fn available_intervals(
    zone: &TimeZone,
    agenda: &AgendaInput<'_>,
    occupancy: &[Interval],
    from: Timestamp,
    until: Timestamp,
    duration_minutes: i64,
) -> Result<Vec<Interval>> {
    if from >= until {
        return Err(SchedulingError::validation(
            "el rango debe contener al menos un día",
        ));
    }

    if !(1..=1440).contains(&duration_minutes) {
        return Err(SchedulingError::validation(
            "la duración debe ser un entero entre 1 y 1440",
        ));
    }

    let mut open: Vec<Interval> = Vec::new();
    let mut date = from.to_zoned(zone.clone()).date();
    let last = until.to_zoned(zone.clone()).date();

    while date <= last {
        for minutes in daily_minutes(date, agenda.rules, agenda.exceptions) {
            let (start, end) = civil_interval_to_utc(
                zone,
                date,
                super::domain::LocalStartMinute::new(minutes.start)?,
                super::domain::LocalEndMinute::new(minutes.end)?,
            )?;

            // La consulta acota el resultado: nada fuera de [from, until).
            let start = start.max(from);
            let end = end.min(until);

            if start < end {
                open.push(Interval { start, end });
            }
        }

        date = date
            .checked_add(Span::new().days(1))
            .map_err(|_| SchedulingError::internal("fecha civil fuera de rango"))?;
    }

    let cuts: Vec<Interval> = agenda
        .blocks
        .iter()
        .filter(|block| block.status == "active")
        .map(|block| -> Result<Interval> {
            Ok(Interval {
                start: Timestamp::from_str(&block.starts_at)
                    .map_err(|_| SchedulingError::internal("bloqueo con instante inválido"))?,
                end: Timestamp::from_str(&block.ends_at)
                    .map_err(|_| SchedulingError::internal("bloqueo con instante inválido"))?,
            })
        })
        .collect::<Result<_>>()?;
    let mut cuts = cuts;
    cuts.extend(merge(occupancy.to_vec()));

    Ok(fitting(
        merge(subtract(merge(open), &cuts)),
        duration_minutes,
    ))
}
