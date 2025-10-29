use std::sync::Mutex;

/// Global mutex to prevent concurrent directory changes across all tests.
/// This must be locked by any test that changes the current working directory
/// to prevent race conditions.
pub static CHDIR_MUTEX: Mutex<()> = Mutex::new(());
