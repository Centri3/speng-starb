//! Module for analyzing `EXE`'s `IMAGE_NT_HEADERS`. Provides `HEADERS`.

use crate::exe::EXE;
use crate::once::OnceCell;
use eyre::Result;
use hashbrown::HashMap;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::ops::Range;

/// Type definition for `IMAGE_SECTION_HEADER`.
pub type NtImageSections = HashMap<String, Range<usize>>;

/// Global variable for `NtImage`. Can also be referenced using `EXE.headers()`
pub static HEADERS: NtImage = NtImage::__define();

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct NtImage {
    optional: OnceCell<NtOptional>,
    sections: OnceCell<NtImageSections>,
}

#[allow(dead_code)]
impl NtImage {
    /// Signature (or e_magic) of DOS headers
    const MZ_SIGNATURE: usize = 0x0usize;
    /// Expected value of `MZ_SIGNATURE`
    const MZ_SIGNATURE_VALUE: &str = "MZ";
    /// Length of the signature
    const MZ_SIGNATURE_LEN: usize = 0x2usize;
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
    const NT_FILE_OPTIONAL_LEN: usize = 0x12usize;
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
    const NT_OPTIONAL_DIRECTORY: usize = 0x88usize;
    /// Length of each section
    const NT_SECTION_LEN: usize = 0x28usize;
    /// Name of the section. Offset from `MZ_NT_ADDRESS`, `NT_FILE`,
    /// `NT_FILE_LEN` and `NT_FILE_OPTIONAL_LEN`
    const NT_SECTION_NAME: usize = 0x0usize;
    /// Size of the section. Offset from `MZ_NT_ADDRESS`, `NT_FILE`,
    /// `NT_FILE_LEN` and `NT_FILE_OPTIONAL_LEN`
    const NT_SECTION_SIZE: usize = 0x10usize;
    /// Start of the section. Offset from `MZ_NT_ADDRESS`, `NT_FILE`,
    /// `NT_FILE_LEN` and `NT_FILE_OPTIONAL_LEN`
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
    fn __read_u32_to_usize(index: usize) -> Result<usize> {
        Ok(EXE.read_to::<u32>(index)? as usize)
    }

    /// Internal function to reduce code repetition. Allows getting any of
    /// `HEADERS`' fields which are wrapped in `OnceCell<T>`.
    #[inline(always)]
    #[instrument(skip(value), level = "trace")]
    fn __get_initialized<T: fmt::Debug + Serialize>(value: &OnceCell<T>) -> Result<&T> {
        value
            .get()
            .ok_or_else(|| eyre!("`HEADERS` was uninitialized, please call `.init()`"))
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self) -> Result<()> {
        info!("Initializing `HEADERS`");

        let nt_base = EXE.read_to::<u32>(Self::MZ_NT_ADDRESS)? as usize;

        // Read DOS e_magic and NT signature
        let mz = EXE.read_to_string(Self::MZ_SIGNATURE, Some(2usize))?;
        let pe = EXE.read_to_string(nt_base, Some(4usize))?;

        // Verify both DOS and NT headers exist. This is actually quite nice.
        (mz == "MZ" && pe == "PE").then_some(()).ok_or_else(|| {
            eyre!("`EXE` does not have valid header signatures: mz = {mz}, pe = {pe}")
        })?;

        self.__init_sections(nt_base)?;

        todo!();
    }

    #[inline]
    #[instrument(skip(self))]
    fn __init_sections(&self, nt_base: usize) -> Result<()> {
        let num_sections = EXE.read_to::<u16>(1)? as usize;

        Ok(())
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
