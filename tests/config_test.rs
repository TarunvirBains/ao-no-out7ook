use ao_no_out7ook::config::{Config, load_from_path};
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
    // Empty config should error or verify defaults if we decide to have them
    // For now, let's verify minimum required fields or result
    let config_content = "";
    temp_file.write_all(config_content.as_bytes()).unwrap();

    // This depends on whether we panic or error on missing fields
    // Assuming we use serde/config defaults or Option types, testing error case here:
    let result = load_from_path(temp_file.path());
    assert!(
        result.is_err(),
        "Should fail on empty config if fields are required"
    );
}
