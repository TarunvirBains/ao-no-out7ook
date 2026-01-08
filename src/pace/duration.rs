/// Duration computation from multiple sources (FR2.4)
pub enum DurationSource {
    Timer { duration_secs: u32 },
    Manual { duration_secs: u32 },
}

pub fn compute_duration(source: DurationSource) -> u32 {
    match source {
        DurationSource::Timer { duration_secs } => duration_secs,
        DurationSource::Manual { duration_secs } => duration_secs,
    }
}

pub fn format_duration(secs: u32) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_hours_and_mins() {
        assert_eq!(format_duration(3665), "1h 1m");
    }

    #[test]
    fn test_format_duration_only_mins() {
        assert_eq!(format_duration(300), "5m");
    }

    #[test]
    fn test_compute_duration_timer() {
        let source = DurationSource::Timer {
            duration_secs: 7200,
        };
        assert_eq!(compute_duration(source), 7200);
    }
}
