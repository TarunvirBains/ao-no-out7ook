use ao_no_out7ook::config::{Config, FocusBlocksConfig, WorkHoursConfig};

#[test]
fn test_work_hours_validation_valid() {
    let config = WorkHoursConfig {
        start: "09:00".to_string(),
        end: "17:00".to_string(),
        timezone: "America/Los_Angeles".to_string(),
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_work_hours_validation_empty() {
    let config = WorkHoursConfig {
        start: "".to_string(),
        end: "".to_string(),
        timezone: "".to_string(),
    };

    // Empty is acceptable (no validation)
    assert!(config.validate().is_ok());
}

#[test]
fn test_work_hours_validation_invalid_format() {
    let config = WorkHoursConfig {
        start: "9am".to_string(),
        end: "5pm".to_string(),
        timezone: "America/Los_Angeles".to_string(),
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid time format")
    );
}

#[test]
fn test_work_hours_validation_invalid_hour() {
    let config = WorkHoursConfig {
        start: "25:00".to_string(),
        end: "17:00".to_string(),
        timezone: "America/Los_Angeles".to_string(),
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_work_hours_validation_start_after_end() {
    let config = WorkHoursConfig {
        start: "17:00".to_string(),
        end: "09:00".to_string(),
        timezone: "America/Los_Angeles".to_string(),
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("start time must be before end time")
    );
}

#[test]
fn test_work_hours_validation_equal_times() {
    let config = WorkHoursConfig {
        start: "09:00".to_string(),
        end: "09:00".to_string(),
        timezone: "America/Los_Angeles".to_string(),
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_focus_blocks_validation_valid() {
    let config = FocusBlocksConfig {
        duration_minutes: 45,
        interval_minutes: 15,
        teams_presence_sync: true,
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_focus_blocks_validation_zero_duration() {
    let config = FocusBlocksConfig {
        duration_minutes: 0,
        interval_minutes: 15,
        teams_presence_sync: true,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duration must be greater than 0")
    );
}

#[test]
fn test_focus_blocks_validation_unusual_interval() {
    let config = FocusBlocksConfig {
        duration_minutes: 45,
        interval_minutes: 17, // Unusual value
        teams_presence_sync: true,
    };

    // Should succeed but print warning (we can't test stderr easily)
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_calls_sub_validators() {
    let mut config = Config::default();
    config.work_hours.start = "25:00".to_string();
    config.work_hours.end = "17:00".to_string();

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_config_get_devops_pat_fallback_to_config() {
    let mut config = Config::default();
    config.devops.pat = Some("test-pat-123".to_string());

    // Should fall back to config field when keyring doesn't have it
    let result = config.get_devops_pat();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-pat-123");
}

#[test]
fn test_config_get_devops_pat_missing() {
    let config = Config::default();

    // Should fail when neither keyring nor config has PAT
    let result = config.get_devops_pat();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("PAT not found"));
}
