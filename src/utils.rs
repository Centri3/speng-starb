//! Contains useful things

use eyre::Report;
use eyre::Result;

/// Internal function to reduce code repetition.
pub fn __report_unreachable_str(msg: impl AsRef<str>) -> Report {
    eyre!("Entered unreachable code: {}", msg.as_ref())
}

/// Internal function to reduce code repetition. Calls
/// `__report_unreachable_str` and wraps it in `Err`.
pub fn __return_unreachable_str(msg: impl AsRef<str>) -> Result<()> {
    Err(__report_unreachable_str(msg))
}

/// Internal function to reduce code repetition. Calls
/// `__report_unreachable_str()` with an empty string.
pub fn __report_unreachable() -> Report {
    __report_unreachable_str("")
}

/// Internal function to reduce code repetition. Calls `__report_unreachable`
/// and wraps it in `Err`.
pub fn __return_unreachable() -> Result<()> {
    Err(__report_unreachable())
}
