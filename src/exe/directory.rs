pub mod import_table;

use self::import_table::ImportTable;

/// Enum containing every entry of `NtDirectory`
#[derive(Debug, Default)]
#[repr(usize)]
#[non_exhaustive]
pub enum NtDirectoryEntries<'a> {
    /// Used for entries which are currently unimplemented. We want this to
    /// return `Index out of bounds` if ever used
    #[default]
    Unused = i32::MAX as _,
    /// Entry for iterating `EXE`'s Import Address Table
    ImportTable(ImportTable<'a>) = 12usize,
}
