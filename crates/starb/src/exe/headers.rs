// TODO: Light documentation

use crate::exe::EXE;
use color_eyre::Help;
use eyre::Result;
use hashbrown::HashMap;
use once_cell::sync::OnceCell;
use std::fmt;
use std::ops::Range;

/// Type definition for `IMAGE_DATA_DIRECTORY`.
pub type NtDataDirectory = [Range<usize>; 16usize];
/// Type definition for `IMAGE_SECTION_HEADER`.
pub type NtImageSections = HashMap<String, Range<usize>>;

/// Global variable for `NtImage`. Can also be referenced using `EXE.headers()`
pub static HEADERS: NtImage = NtImage::__define();

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct NtImage {
    optional: OnceCell<NtOptional>,
    sections: OnceCell<NtImageSections>,
}

impl NtImage {
    /// Signature (or e_magic) of DOS headers
    const MZ_SIGNATURE: usize = 0x0usize;
    /// Signature of NT headers
    const NT_SIGNATURE: usize = 0x170usize;
    /// Entry point of program
    const NT_ENTRY_POINT: usize = 0x198usize;
    /// Length of each entry in `IMAGE_DATA_DIRECTORY`
    const NT_DIRECTORY_ENTRY_LEN: usize = 0x8usize;
    /// Where the data directory starts in the exe
    const NT_DIRECTORY: usize = 0x1f8usize;
    /// How many entries the data directory can contain
    const NT_DIRECTORY_NUM: usize = 0x10usize;
    /// Length of each section in `IMAGE_SECTION_HEADER`
    const NT_SECTION_LEN: usize = 0x8usize;
    /// Where sections start in the exe
    const NT_SECTIONS: usize = 0x278usize;
    /// How many sections this executable contains
    const NT_SECTIONS_NUM: usize = 0x176usize;

    /// Internal function to define `HEADERS`.
    #[inline(always)]
    const fn __define() -> Self {
        Self {
            optional: OnceCell::new(),
            sections: OnceCell::new(),
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn __read_u32_to_usize(index: usize) -> Result<usize> {
        Ok(EXE.read_to::<u32>(index)? as usize)
    }

    /// Internal function to reduce code repetition. Allows getting any of
    /// `HEADERS`' fields which are wrapped in `OnceCell<T>`.
    #[inline(always)]
    #[instrument(skip(value), level = "trace")]
    fn __get_initialized<T: fmt::Debug>(value: &OnceCell<T>) -> Result<&T> {
        value
            .get()
            .ok_or_else(|| eyre!("`HEADERS` was uninitialized, please call `.init()`"))
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self) -> Result<()> {
        info!("Initializing `HEADERS`");

        // Read DOS e_magic and NT signature
        let mz = EXE.read_to_string(Self::MZ_SIGNATURE, Some(2usize))?;
        let pe = EXE.read_to_string(Self::NT_SIGNATURE, Some(4usize))?;

        // Verify both DOS and NT headers exist. This is actually quite nice.
        (mz != "MZ" || pe != "PE")
            .then_some(())
            .ok_or_else(|| eyre!("`EXE` does not have valid headers: mz = {mz}, pe = {pe}"))?;

        Self::__init_optional()?;

        Self::__init_sections()?;

        Ok(())
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn __init_optional() -> Result<NtOptional> {
        let entry_point = Self::__read_u32_to_usize(Self::NT_ENTRY_POINT)?;
        let directories = Self::__get_directories()?;

        Ok(NtOptional::new(entry_point, directories))
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn __get_directories() -> Result<NtDataDirectory> {
        // TODO: We should create a struct for this, not a type definition
        Ok(NtDataDirectory::default())
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn __init_sections() -> Result<NtImageSections> {
        Ok(NtImageSections::default())
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn optional(&self) -> Result<&NtOptional> {
        Self::__get_initialized(&self.optional)
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn sections(&self) -> Result<&NtImageSections> {
        Self::__get_initialized(&self.sections)
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct NtOptional {
    entry_point: usize,
    directory: NtDataDirectory,
}

impl NtOptional {
    #[inline]
    pub const fn new(entry_point: usize, directory: NtDataDirectory) -> Self {
        Self {
            entry_point,
            directory,
        }
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn entry_point(&self) -> usize {
        self.entry_point
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn directory(&self) -> &NtDataDirectory {
        &self.directory
    }
}
