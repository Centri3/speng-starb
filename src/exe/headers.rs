//! Module for analyzing `EXE`'s `IMAGE_NT_HEADERS`. Provides `HEADERS`.

use crate::exe::EXE;
use crate::once::deserialize;
use crate::once::serialize;
use eyre::Result;
use hashbrown::HashMap;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde::Serialize;
use std::any;
use std::fmt;
use std::ops::Range;

/// Type definition for `IMAGE_SECTION_HEADER`.
pub type NtImageSections = HashMap<String, Range<usize>>;

/// Global variable for `NtImage`. Can also be referenced using `EXE.headers()`
pub static HEADERS: NtImage = NtImage::__define();

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct NtImage {
    #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
    optional: OnceCell<NtOptional>,
    #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
    sections: OnceCell<NtImageSections>,
}

impl NtImage {
    /// Signature (or e_magic) of DOS headers
    const MZ_MAGIC: usize = 0x0usize;
    /// Expected value of `MZ_SIGNATURE`
    const MZ_MAGIC_VALUE: &str = "MZ";
    /// Length of the signature
    const MZ_MAGIC_LEN: usize = 0x2usize;
    /// Address of `IMAGE_NT_HEADERS`
    const MZ_NT_ADDRESS: usize = 0x3cusize;
    /// Signature of NT headers. Offset from `MZ_NT_ADDRESS`
    const NT_SIGNATURE: usize = 0x0usize;
    /// Expected value of `NT_SIGNATURE`
    const NT_SIGNATURE_VALUE: &str = "PE";
    /// Length of the signature
    const NT_SIGNATURE_LEN: usize = 0x4usize;
    /// Address where the `IMAGE_FILE_HEADER` starts. Offset from
    /// `MZ_NT_ADDRESS`
    const NT_FILE: usize = 0x4usize;
    /// Length of the `IMAGE_FILE_HEADER`
    const NT_FILE_LEN: usize = 0x14usize;
    /// Number of sections in `EXE`. Offset from `MZ_NT_ADDRESS` and
    /// `NT_FILE`
    const NT_FILE_SECTIONS_NUM: usize = 0x2usize;
    /// Size of the optional header in `EXE`. Offset from `MZ_NT_ADDRESS` and
    /// `NT_FILE`
    const NT_FILE_OPTIONAL_LEN: usize = 0x10usize;
    /// Expected value of `NT_FILE_OPTIONAL_LEN`
    const NT_FILE_OPTIONAL_LEN_VALUE: usize = 0xf0usize;
    /// Address where the `IMAGE_OPTIONAL_HEADER` starts. Offset from
    /// `MZ_NT_ADDRESS`
    const NT_OPTIONAL: usize = 0x18usize;
    /// Magic. Offset from `MZ_NT_ADDRESS` and `NT_OPTIONAL`
    const NT_OPTIONAL_MAGIC: usize = 0x0usize;
    /// Expected value of `NT_OPTIONAL_MAGIC` if `EXE` is a 32-bit executable
    const NT_OPTIONAL_MAGIC_32: usize = 0x10busize;
    /// Expected value of `NT_OPTIONAL_MAGIC` if `EXE` is a 64-bit executable
    const NT_OPTIONAL_MAGIC_64: usize = 0x20busize;
    /// Entry point of the program. Offset from `MZ_NT_ADDRESS` and
    /// `NT_OPTIONAL`
    const NT_OPTIONAL_ENTRY_POINT: usize = 0x28usize;
    /// Number of entries in `IMAGE_DATA_DIRECTORY`. Offset from `MZ_NT_ADDRESS`
    /// and `NT_OPTIONAL`
    const NT_OPTIONAL_DIRECTORY_ENTRY_NUM: usize = 0x84usize;
    /// Expected value of `NT_OPTIONAL_DIRECTORY_ENTRY_NUM`. Quite a mouthful.
    const NT_OPTIONAL_DIRECTORY_ENTRY_NUM_VALUE: usize = 0x10usize;
    /// Length of each entry in `IMAGE_DATA_DIRECTORY`. Offset from
    /// `MZ_NT_ADDRESS` and `NT_OPTIONAL`
    const NT_OPTIONAL_DIRECTORY_ENTRY_LEN: usize = 0x8usize;
    /// Address where the `IMAGE_DATA_DIRECTORY` begins. Offset from
    /// `MZ_NT_ADDRESS` and `NT_OPTIONAL`
    const NT_OPTIONAL_DIRECTORY: usize = 0x70usize;
    /// Length of each section
    const NT_SECTION_LEN: usize = 0x28usize;
    /// Name of the section. Offset from `MZ_NT_ADDRESS`, `NT_OPTIONAL` and
    /// `NT_FILE_OPTIONAL_LEN`
    const NT_SECTION_NAME: usize = 0x0usize;
    /// Size of the section. Offset from `MZ_NT_ADDRESS`, `NT_OPTIONAL` and
    /// `NT_FILE_OPTIONAL_LEN`
    const NT_SECTION_SIZE: usize = 0x10usize;
    /// Start of the section. Offset from `MZ_NT_ADDRESS`, `NT_OPTIONAL` and
    /// `NT_FILE_OPTIONAL_LEN`
    const NT_SECTION: usize = 0x14usize;

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
    fn __read_u16_to_usize(index: usize) -> Result<usize> {
        Ok(EXE.read_to::<u16>(index)? as usize)
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn __read_u32_to_usize(index: usize) -> Result<usize> {
        Ok(EXE.read_to::<u32>(index)? as usize)
    }

    /// Internal function to reduce code repetition. Allows getting any of
    /// `HEADERS`' fields which are wrapped in `OnceCell<T>`.
    #[inline(always)]
    #[instrument(skip(value), level = "trace", fields(T = any::type_name::<T>()))]
    fn __get_initialized<T: fmt::Debug + Serialize>(value: &OnceCell<T>) -> Result<&T> {
        value
            .get()
            .ok_or_else(|| eyre!("`HEADERS` was uninitialized, please call `.init()`"))
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self) -> Result<()> {
        info!("Initializing `HEADERS`");

        // Base of `IMAGE_NT_HEADERS`
        let nt_base = Self::__read_u32_to_usize(Self::MZ_NT_ADDRESS)?;
        // Base of `IMAGE_FILE_HEADER`
        let nt_file = nt_base + Self::NT_FILE;
        // Base of `IMAGE_OPTIONAL_HEADER`
        let nt_optional = nt_base + Self::NT_OPTIONAL;
        // Base of `IMAGE_DATA_DIRECTORY` in `IMAGE_OPTIONAL_HEADER`
        let nt_directory = nt_optional + Self::NT_OPTIONAL_DIRECTORY;

        // Read DOS e_magic and NT signature
        let mz = EXE.read_to_string(Self::MZ_MAGIC, Some(Self::MZ_MAGIC_LEN))?;
        let pe = EXE.read_to_string(nt_base, Some(Self::NT_SIGNATURE_LEN))?;

        // Verify both DOS and NT headers exist. Will fail if either are invalid
        (mz == Self::MZ_MAGIC_VALUE && pe == Self::NT_SIGNATURE_VALUE)
            .then_some(())
            .ok_or_else(|| {
                eyre!("`EXE` does not have valid header signatures: mz = {mz}, pe = {pe}")
            })?;

        self.__init_optional(nt_optional)?;

        // Base of `IMAGE_SECTION_HEADER`
        let nt_sections =
            nt_optional + Self::__read_u16_to_usize(nt_file + Self::NT_FILE_OPTIONAL_LEN)?;

        self.__init_sections(nt_sections)?;

        todo!();
    }

    #[inline]
    #[instrument(skip(self), level = "trace")]
    fn __init_optional(&self, nt_optional: usize) -> Result<()> {
        let entry_point = nt_optional + Self::__read_u32_to_usize(Self::NT_OPTIONAL_ENTRY_POINT)?;
        let directory = self.__get_directory(nt_optional)?;

        self.optional
            .set(NtOptional::new(entry_point, directory))
            .map_err(|_| eyre!("`HEADERS` was already initialized"))?;

        trace!("Initialized `HEADERS.optional`");

        Ok(())
    }

    #[inline]
    #[instrument(skip(self), level = "trace")]
    fn __get_directory(&self, nt_optional: usize) -> Result<()> {
        // TODO:
        Ok(())
    }

    #[inline]
    #[instrument(skip(self), level = "trace")]
    fn __init_sections(&self, nt_sections: usize) -> Result<()> {
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

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct NtOptional {
    entry_point: usize,
    directory: (),
}

impl NtOptional {
    #[inline]
    pub const fn new(entry_point: usize, directory: ()) -> Self {
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
    pub fn directory(&self) -> &() {
        &self.directory
    }
}
