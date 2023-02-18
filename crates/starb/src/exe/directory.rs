//! Module for analyzing `EXE`'s `IMAGE_DATA_DIRECTORY`.

use std::fmt;
use std::ops::Range;

#[derive(Debug)]
#[non_exhaustive]
pub struct NtDataDirectoryEntry<E>
where
    E: fmt::Debug + NtDataDirectoryEntryAcceptable,
{
    scope: Range<usize>,
    data: Box<E>,
}

impl<E> NtDataDirectoryEntry<E>
where
    E: fmt::Debug + NtDataDirectoryEntryAcceptable,
{
    pub fn scope(&self) -> &Range<usize> {
        &self.scope
    }

    pub fn data(&self) -> &Box<E> {
        &self.data
    }
}

// TODO: Wtf is this name?
pub trait NtDataDirectoryEntryAcceptable {}
