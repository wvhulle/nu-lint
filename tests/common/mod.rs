#![allow(dead_code)]
use std::sync::Mutex;
use tempfile::TempDir;

/// Global mutex to prevent concurrent directory changes across all tests.
/// This must be locked by any test that changes the current working directory
/// to prevent race conditions.
pub static CHDIR_MUTEX: Mutex<()> = Mutex::new(());

/// Helper function to run a test in a temporary directory with proper cleanup.
/// Ensures the current directory is restored even if the test panics.
pub fn with_temp_dir<F>(f: F)
where
    F: FnOnce(&TempDir),
{
    let _guard = CHDIR_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Use catch_unwind to ensure directory is restored even if the test panics
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::env::set_current_dir(temp_dir.path()).unwrap();
        f(&temp_dir);
    }));

    // Always restore directory, even if test panics
    std::env::set_current_dir(original_dir).unwrap();

    // Re-panic if the test failed
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
