use eyre::Result;
use hashbrown::HashMap;
use once_cell::sync::OnceCell;
use std::fmt;
use std::ops::Range;

/// Type definition for `IMAGE_DATA_DIRECTORY`.
pub type NtDataDirectory = Vec<Range<usize>>;
/// Type definition for `IMAGE_SECTION_HEADER`.
pub type NtImageSections = HashMap<String, Range<usize>>;

/// Global variable for `NtImage`. Can also be referenced using `EXE.headers()`
pub static HEADERS: NtImage = NtImage::__define();

#[derive(Debug, Default)]
pub struct NtImage {
    optional: OnceCell<NtOptional>,
    sections: OnceCell<NtImageSections>,
}

impl NtImage {
    /// Internal function to define `HEADERS`.
    #[inline(always)]
    const fn __define() -> Self {
        Self {
            optional: OnceCell::new(),
            sections: OnceCell::new(),
        }
    }

    /// Internal function to reduce code repetition. Allows getting any of
    /// `HEADER`'s fields which are wrapped in `OnceCell<T>`.
    #[inline(always)]
    #[instrument(skip(value), level = "trace")]
    fn __get_expect_uninitialized<T: fmt::Debug>(value: &OnceCell<T>) -> Result<&T> {
        value
            .get()
            .ok_or(eyre!("`HEADERS` was uninitialized, please call `.init()`"))
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self) -> Result<()> {
        todo!();
    }
}

#[derive(Debug, Default)]
pub struct NtOptional {
    entry_point: usize,
    directory: NtDataDirectory,
}
