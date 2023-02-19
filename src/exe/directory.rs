use self::functions::Functions;
use self::imports::Imports;

pub mod functions;
pub mod imports;

/// Enum containing every entry of `NtDirectory`
#[derive(Debug, Default)]
#[repr(usize)]
#[non_exhaustive]
pub enum NtDirectoryEntries {
    /// Used for entries which are currently unimplemented. We want this to
    /// return `Index out of bounds` if ever used
    #[default]
    Unused = i32::MAX as _,
    /// Entry for iterating `EXE`'s `IMAGE_IMPORT_DESCRIPTOR`s
    Imports(Imports) = 1usize,
    /// Entry for iterating `EXE`'s `IMAGE_RUNTIME_FUNCTION_ENTRY` array. This
    /// is normally used for exception handling, but is instead used for
    /// patching here.
    Functions(Functions) = 3usize,
}
