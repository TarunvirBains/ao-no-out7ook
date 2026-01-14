use ao_no_out7ook::state::{State, with_state_lock};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_state_creation() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join("state.json");
    let _lock_path = dir.path().join("state.lock");

    // Initialize empty state
    let state = State::default();
    state.save(&state_path).unwrap();

    // Load back
    let loaded = State::load(&state_path).unwrap();
    assert!(loaded.current_task.is_none());
    assert_eq!(loaded.version, "1.0.0");
}

#[test]
fn test_concurrent_lock() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join("state.json");
    let lock_path = dir.path().join("state.lock");

    // Create initial state
    State::default().save(&state_path).unwrap();

    let lock_path_clone = lock_path.clone();
    let state_path_clone = state_path.clone();

    // Spawn a thread that holds the lock for 500ms
    let handle = thread::spawn(move || {
        with_state_lock(&lock_path_clone, &state_path_clone, |state| {
            state.version = "locked".to_string();
            thread::sleep(Duration::from_millis(500));
            Ok(())
        })
        .unwrap();
    });

    // Give thread time to acquire lock
    thread::sleep(Duration::from_millis(100));

    // Attempt to acquire lock - should block until thread finishes
    let start = std::time::Instant::now();
    with_state_lock(&lock_path, &state_path, |state| {
        // When we get here, version should be "locked"
        assert_eq!(state.version, "locked");
        state.version = "updated".to_string();
        Ok(())
    })
    .unwrap();

    assert!(
        start.elapsed().as_millis() >= 400,
        "Should have waited for lock"
    );

    handle.join().unwrap();

    // Verify final state
    let final_state = State::load(&state_path).unwrap();
    assert_eq!(final_state.version, "updated");
}
