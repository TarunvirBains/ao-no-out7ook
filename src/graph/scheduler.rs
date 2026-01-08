use crate::config::WorkHoursConfig;
use crate::graph::models::{CalendarEvent, DateTimeTimeZone};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Timelike, Utc};

/// Round to next 15-minute interval (:00, :15, :30, :45)
pub fn round_to_next_interval(time: DateTime<Utc>) -> DateTime<Utc> {
    let minute = time.minute();
    let next_interval = match minute {
        0..=14 => 15,
        15 => 15, // Already aligned
        16..=30 => 30,
        31..=45 => 45,
        46..=59 => 0, // Roll to next hour
        _ => unreachable!(),
    };

    if next_interval == 0 {
        // Roll to next hour
        time.with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
            + Duration::hours(1)
    } else if minute <= next_interval {
        time.with_minute(next_interval)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    } else {
        time.with_minute(next_interval)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }
}

/// Parse DateTime from event's DateTimeTimeZone
fn parse_event_time(dt: &DateTimeTimeZone) -> Result<DateTime<Utc>> {
    let datetime_str = &dt.date_time;

    // Try parsing as RFC3339 first
    if let Ok(parsed) = DateTime::parse_from_rfc3339(datetime_str) {
        return Ok(parsed.with_timezone(&Utc));
    }

    // Try parsing without timezone (assume UTC)
    let formats = ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M:%S%.f"];

    for format in &formats {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(datetime_str, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    anyhow::bail!("Failed to parse datetime: {}", datetime_str)
}

/// Find gaps between events (free time slots)
pub fn find_gaps(
    events: &[CalendarEvent],
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>> {
    if events.is_empty() {
        return Ok(vec![(start_time, end_time)]);
    }

    // Parse and sort events by start time, filtering out events that end before our search window
    let mut sorted_events: Vec<(DateTime<Utc>, DateTime<Utc>)> = events
        .iter()
        .filter_map(|e| {
            let event_start = parse_event_time(&e.start).ok()?;
            let event_end = parse_event_time(&e.end).ok()?;

            // Skip events that end before or at our start_time (they're in the past)
            if event_end <= start_time {
                return None;
            }

            // Skip events that start after our end_time (they're too far in future)
            if event_start >= end_time {
                return None;
            }

            Some((event_start, event_end))
        })
        .collect();

    sorted_events.sort_by_key(|(start, _)| *start);

    let mut gaps = Vec::new();
    let mut current = start_time;

    for (event_start, event_end) in sorted_events {
        // If there's a gap before this event
        if current < event_start {
            gaps.push((current, event_start));
        }
        // Move past this event
        current = current.max(event_end);
    }

    // Gap after last event
    if current < end_time {
        gaps.push((current, end_time));
    }

    Ok(gaps)
}

/// FR3.7: Find next available slot for Focus Block
pub fn find_next_slot(
    events: &[CalendarEvent],
    now: DateTime<Utc>,
    duration_mins: u32,
    work_hours: &WorkHoursConfig,
) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    // Parse work hours
    let work_start = NaiveTime::parse_from_str(&work_hours.start, "%H:%M")
        .context("Invalid work hours start time format")?;
    let work_end = NaiveTime::parse_from_str(&work_hours.end, "%H:%M")
        .context("Invalid work hours end time format")?;

    // Round current time to next interval
    let search_start = round_to_next_interval(now);

    // Try today first
    let mut search_day = search_start.date_naive();

    // Try up to 7 days in the future
    for _ in 0..7 {
        let day_start = search_day
            .and_time(work_start)
            .and_local_timezone(Utc)
            .single()
            .unwrap_or_else(|| Utc.from_utc_datetime(&search_day.and_time(work_start)));

        let day_end = search_day
            .and_time(work_end)
            .and_local_timezone(Utc)
            .single()
            .unwrap_or_else(|| Utc.from_utc_datetime(&search_day.and_time(work_end)));

        // For today, start from current time (rounded)
        let actual_start = if search_day == now.date_naive() {
            search_start.max(day_start)
        } else {
            day_start
        };

        // Find gaps in this day
        let gaps = find_gaps(events, actual_start, day_end)?;

        // Find first gap that fits duration
        for (gap_start, gap_end) in gaps {
            let gap_duration_mins = (gap_end - gap_start).num_minutes() as u32;

            if gap_duration_mins >= duration_mins {
                // Use gap_start if it's already aligned to 15-min, otherwise round it
                let slot_start = if gap_start.minute() % 15 == 0 {
                    gap_start
                } else {
                    round_to_next_interval(gap_start)
                };
                let slot_end = slot_start + Duration::minutes(duration_mins as i64);

                // Ensure slot doesn't exceed gap or work hours
                if slot_end <= gap_end && slot_end <= day_end {
                    return Ok((slot_start, slot_end));
                }
            }
        }

        // Try next day
        search_day = search_day.succ_opt().context("Date overflow")?;
    }

    anyhow::bail!("Could not find available slot in next 7 days")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn default_work_hours() -> WorkHoursConfig {
        WorkHoursConfig {
            start: "08:30".to_string(),
            end: "17:00".to_string(),
            timezone: "UTC".to_string(),
        }
    }

    fn mock_event_utc(
        year: i32,
        month: u32,
        day: u32,
        start_hour: u32,
        start_min: u32,
        end_hour: u32,
        end_min: u32,
    ) -> CalendarEvent {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let start = date
            .and_hms_opt(start_hour, start_min, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let end = date
            .and_hms_opt(end_hour, end_min, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        CalendarEvent {
            id: Some("test".to_string()),
            subject: "Test Event".to_string(),
            start: DateTimeTimeZone {
                date_time: start.to_rfc3339(),
                time_zone: "UTC".to_string(),
            },
            end: DateTimeTimeZone {
                date_time: end.to_rfc3339(),
                time_zone: "UTC".to_string(),
            },
            body: None,
            categories: vec![],
            extended_properties: None,
        }
    }

    #[test]
    fn test_round_to_next_interval() {
        let time = Utc.with_ymd_and_hms(2026, 1, 8, 9, 7, 0).unwrap();
        let rounded = round_to_next_interval(time);
        assert_eq!(rounded.hour(), 9);
        assert_eq!(rounded.minute(), 15);

        let time = Utc.with_ymd_and_hms(2026, 1, 8, 9, 15, 0).unwrap();
        let rounded = round_to_next_interval(time);
        assert_eq!(rounded.minute(), 15); // Already aligned

        let time = Utc.with_ymd_and_hms(2026, 1, 8, 9, 47, 0).unwrap();
        let rounded = round_to_next_interval(time);
        assert_eq!(rounded.hour(), 10);
        assert_eq!(rounded.minute(), 0);
    }

    #[test]
    fn test_find_next_slot_empty_calendar() {
        let events = vec![];
        let now = Utc.with_ymd_and_hms(2026, 1, 8, 9, 7, 0).unwrap();
        let work_hours = default_work_hours();

        let (start, end) = find_next_slot(&events, now, 45, &work_hours).unwrap();

        assert_eq!(start.hour(), 9);
        assert_eq!(start.minute(), 15);
        assert_eq!(end.hour(), 10);
        assert_eq!(end.minute(), 0);
    }

    #[test]
    fn test_find_next_slot_with_gap() {
        // Events: 9-10am, 11-12pm (gap: 10-11am)
        let events = vec![
            mock_event_utc(2026, 1, 8, 9, 0, 10, 0),
            mock_event_utc(2026, 1, 8, 11, 0, 12, 0),
        ];
        // Start at 9:30am (during first event), should find gap at 10am
        let now = Utc.with_ymd_and_hms(2026, 1, 8, 9, 30, 0).unwrap();
        let work_hours = default_work_hours();

        let (start, end) = find_next_slot(&events, now, 45, &work_hours).unwrap();

        // Should find gap at 10:00-10:45
        assert_eq!(start.hour(), 10);
        assert_eq!(start.minute(), 0);
        assert_eq!(end.hour(), 10);
        assert_eq!(end.minute(), 45);
    }

    #[test]
    fn test_find_next_slot_skip_small_gap() {
        // Events with only 30-min gap (10:00-10:30)
        let events = vec![
            mock_event_utc(2026, 1, 8, 9, 0, 10, 0),
            mock_event_utc(2026, 1, 8, 10, 30, 12, 0),
        ];
        let now = Utc.with_ymd_and_hms(2026, 1, 8, 8, 30, 0).unwrap();
        let work_hours = default_work_hours();

        let (start, _) = find_next_slot(&events, now, 45, &work_hours).unwrap();

        // Should skip 30-min gap and use time after 12pm
        assert!(start.hour() >= 12);
    }

    #[test]
    fn test_rollover_to_next_day() {
        // Fully booked today (8:30am-5pm)
        let events = vec![mock_event_utc(2026, 1, 8, 8, 30, 17, 0)];
        let now = Utc.with_ymd_and_hms(2026, 1, 8, 16, 30, 0).unwrap();
        let work_hours = default_work_hours();

        let (start, _) = find_next_slot(&events, now, 45, &work_hours).unwrap();

        // Should be next day at 8:30am
        assert_eq!(start.day(), 9); // Next day
        assert_eq!(start.hour(), 8);
        assert_eq!(start.minute(), 30);
    }

    #[test]
    fn test_find_gaps_empty() {
        let events = vec![];
        let start = Utc.with_ymd_and_hms(2026, 1, 8, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 8, 17, 0, 0).unwrap();

        let gaps = find_gaps(&events, start, end).unwrap();

        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].0, start);
        assert_eq!(gaps[0].1, end);
    }

    #[test]
    fn test_find_gaps_multiple() {
        let events = vec![
            mock_event_utc(2026, 1, 8, 9, 0, 10, 0),
            mock_event_utc(2026, 1, 8, 11, 0, 12, 0),
            mock_event_utc(2026, 1, 8, 14, 0, 15, 0),
        ];
        let start = Utc.with_ymd_and_hms(2026, 1, 8, 8, 30, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 8, 17, 0, 0).unwrap();

        let gaps = find_gaps(&events, start, end).unwrap();

        // Should have 4 gaps: before first, between 1-2, between 2-3, after last
        assert_eq!(gaps.len(), 4);
        assert_eq!(gaps[0].0.hour(), 8); // Before first event
        assert_eq!(gaps[1].0.hour(), 10); // Between first and second
        assert_eq!(gaps[2].0.hour(), 12); // Between second and third
        assert_eq!(gaps[3].0.hour(), 15); // After last event
    }
}
