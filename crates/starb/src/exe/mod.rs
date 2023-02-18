//! Module for analyzing and patching `SpaceEngine.exe`

pub mod directory;
pub mod headers;

// Some re-exports
pub use self::headers::HEADERS;

// We re-export this so it doesn't matter
#[allow(clippy::module_inception)]
pub mod exe;
pub use self::exe::*;
