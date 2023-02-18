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
    // TODO: We should read sections/directories/whatever using constants defined
    // here, as it's just easier to read

    /// Internal function to define `HEADERS`.
    #[inline(always)]
    const fn __define() -> Self {
        Self {
            optional: OnceCell::new(),
            sections: OnceCell::new(),
        }
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

    #[inline(always)]
    #[instrument(skip(self))]
    fn __get_optional(&self) -> Result<NtOptional> {
        let entry_point = EXE.read_to::<u32>(0x198usize)? as usize;
        let directory = self.__get_directory()?;

        Ok(NtOptional::new(entry_point, directory))
    }

    #[inline(always)]
    #[instrument(skip(self))]
    fn __get_directory(&self) -> Result<NtDataDirectory> {
        trace!("Getting data directory");

        let mut directory = NtDataDirectory::default();

        // FIXME: Is this really redunant? Or just bad design on my part?
        // FIXME: Entries can be (NULL, NULL). We should catch this
        #[allow(clippy::redundant_clone)]
        for (i, _) in directory.clone().iter().enumerate() {
            let start = EXE.read_to::<u32>(0x1f8usize + i * 0x8usize)? as usize;
            let range = start..start + EXE.read_to::<u32>(0x1fcusize + i * 0x8usize)? as usize;

            directory[i] = range;
        }

        // TODO: Remove later
        info!("{:x?}", directory);

        Ok(directory)
    }

    // TODO: This should be refactored. Kinda ugly
    #[inline(always)]
    #[instrument(skip(self))]
    fn __get_sections(&self) -> Result<NtImageSections> {
        trace!("Getting sections");

        let mut sections = NtImageSections::with_capacity(EXE.read_to::<u16>(0x176usize)? as _);

        for i in 0usize..sections.capacity() {
            let base = 0x278usize + i * 0x28usize;

            // Read name of section
            let name = EXE.read_to_string(base, Some(8usize))?;

            // Read range of section
            let start = EXE.read_to::<u32>(base + 0x14usize)? as usize;
            let range = start..start + EXE.read_to::<u32>(base + 0x10usize)? as usize;

            info!(section = name, ?range, "Found section");

            // Insert name of section and range of section
            sections.insert(name, range);
        }

        Ok(sections)
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self) -> Result<()> {
        info!("Initializing `HEADERS`");

        // Check whether DOS header exists. If it does, we jump over it
        if EXE.read_to_string(0usize, Some(2usize))? != "MZ" {
            return Err(eyre!("`HEADERS` does not contain DOS stub")
                .suggestion("Select a file that is a portable executable next time"));
        }

        self.__get_optional()?;

        self.__get_sections()?;

        todo!();
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
