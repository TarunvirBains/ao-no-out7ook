use ao_no_out7ook::config::load_from_path;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_config_valid() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let config_content = r#"
        [devops]
        organization = "test_org"
        project = "test_proj"
        
        [work_hours]
        start = "09:00"
        end = "17:00"
        timezone = "UTC"
    "#;
    temp_file.write_all(config_content.as_bytes()).unwrap();

    let config = load_from_path(temp_file.path()).expect("Failed to load valid config");

    assert_eq!(config.devops.organization, "test_org");
    assert_eq!(config.devops.project, "test_proj");
    assert_eq!(config.work_hours.start, "09:00");
}

#[test]
fn test_load_config_defaults() {
    let mut temp_file = NamedTempFile::new().unwrap();
    // Empty config should load with defaults
    let config_content = "";
    temp_file.write_all(config_content.as_bytes()).unwrap();

    let config = load_from_path(temp_file.path()).expect("Should load with defaults");

    // Verify default values are applied
    assert_eq!(config.devops.organization, "");
    assert_eq!(config.devops.project, "");
    // WorkHoursConfig Default has these values (see config.rs:45-51)
    assert_eq!(config.work_hours.start, "");
    assert_eq!(config.work_hours.end, "");
    assert_eq!(config.work_hours.timezone, "");
    assert_eq!(config.graph.tenant_id, "common");
    assert_eq!(config.focus_blocks.duration_minutes, 45);
}
