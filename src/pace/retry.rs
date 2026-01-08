use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;

/// Retry an operation with exponential backoff (FR2.8)
pub fn with_retry<F, T>(operation: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let mut attempt = 0;
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                let backoff_ms = 2_u64.pow(attempt) * 100; // 100ms, 200ms, 400ms, 800ms...
                eprintln!(
                    "⚠ API call failed (attempt {}/{}): {}. Retrying in {}ms...",
                    attempt + 1,
                    max_retries,
                    e,
                    backoff_ms
                );
                sleep(Duration::from_millis(backoff_ms));
                attempt += 1;
            }
            Err(e) => {
                eprintln!("✗ API call failed after {} attempts", max_retries + 1);
                return Err(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_succeeds_on_first_attempt() {
        let result = with_retry(|| Ok::<i32, anyhow::Error>(42), 3);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_retry_succeeds_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    anyhow::bail!("Simulated failure")
                } else {
                    Ok(42)
                }
            },
            3,
        );

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_exhausts_attempts() {
        let result: Result<i32> = with_retry(|| anyhow::bail!("Always fails"), 2);
        assert!(result.is_err());
    }
}
