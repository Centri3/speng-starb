#[derive(Debug)]
pub struct Import {}

/// Enum containing every entry of `NtDirectory`
#[derive(Debug, Default)]
#[repr(usize)]
#[non_exhaustive]
pub enum NtDirectoryEntries {
    #[default]
    /// Used for entries which are currently unimplemented. We want this to
    /// return `Index out of bounds` if ever used
    Unused = i32::MAX as _,
    /// Entry for iterating `EXE`'s Import Address Table
    ImportTable(Vec<Import>) = 12usize,
}
