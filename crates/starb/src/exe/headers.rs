use super::EXE;
use eyre::Result;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::ops::Range;

pub type NtImageSections = HashMap<String, Range<usize>>;

/// Global variable for `NtImage`.
pub static HEADERS: NtImage = NtImage {
    entry_point: OnceCell::new(),
    sections: OnceCell::new(),
};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct NtImage {
    entry_point: OnceCell<usize>,
    sections: OnceCell<NtImageSections>,
}

impl NtImage {
    const NT_START: usize = 0x170usize;
    const NT_ENTRY_POINT: usize = 0x28usize;
    const NT_SECTIONS_START: usize = 0x278usize;
    const NT_SECTIONS_SIZE: usize = 0x28usize;
    const NT_SECTION_NAME: usize = 0x0usize;
    const NT_SECTION_START: usize = 0x14usize;
    const NT_SECTION_SIZE: usize = 0x10usize;

    /// Internal function to reduce code repetition. Read a u32 at `index` then
    /// cast to `usize`
    #[inline(always)]
    #[instrument(skip(self), level = "trace")]
    fn __read_u32_to_usize(&self, index: usize) -> Result<usize> {
        EXE.read_to::<u32>(index).map(|o| o as usize)
    }

    /// Extracted from `.init()`. Get the section which starts at `index`.
    #[inline(always)]
    #[instrument(skip(self), level = "trace")]
    fn __get_section(&self, index: usize) -> Result<(String, Range<usize>)> {
        let name = EXE.read_to_string(index + Self::NT_SECTION_NAME, Some(8usize))?;
        let start = self.__read_u32_to_usize(index + Self::NT_SECTION_START)?;
        let end = start + self.__read_u32_to_usize(index + Self::NT_SECTION_SIZE)?;

        if name.is_empty() {
            info!("Name was empty, probably reached end of sections");
        }

        Ok((name, start..end))
    }

    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self) -> Result<()> {
        info!("Initializing `HEADERS`");

        // Make sure we're looking at `IMAGE_NT_HEADERS64`
        (EXE.read_to_string(Self::NT_START, Some(4usize))? == "PE")
            .then_some(())
            .ok_or(eyre!("`EXE` has invalid/no headers"))?;

        let entry_point = self.__read_u32_to_usize(Self::NT_ENTRY_POINT)?;
        let mut sections = HashMap::new();

        // There can only be, at max, 65536 sections in an .exe. Though I doubt there'd
        // be more than 10 in `SpaceEngine.exe`, as less sections is more efficient
        for i in 0usize..u16::MAX as usize {
            let section =
                self.__get_section(Self::NT_SECTIONS_START + i * Self::NT_SECTIONS_SIZE)?;

            if section.0.is_empty() {
                break;
            }

            info!(?section, "Found section");

            if sections.insert(section.clone().0, section.1).is_some() {
                return Err(eyre!("Header already existed: `{}`", section.0));
            }
        }

        self.entry_point
            .set(entry_point)
            .unwrap_or_else(|_| panic!("`HEADERS` was already initialized"));
        // We don't need to use `.unwrap_or_else()` again, as we know it hasn't been
        // initialized if we got here (still have to unwrap, though)
        self.sections.set(sections).unwrap();

        Ok(())
    }

    /// Internal function to reduce code repetition. Returns `self.entry_point`,
    /// or panics if it was uninitialized.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn entry_point(&self) -> usize {
        *self
            .entry_point
            .get()
            .expect("`HEADERS` was uninitialized, please call `.init()`")
    }

    /// Internal function to reduce code repetition. Returns `self.sections`, or
    /// panics if it was uninitialized.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn sections(&self) -> NtImageSections {
        self.sections
            .get()
            .expect("`HEADERS` was uninitialized, please call `.init()`")
            .clone()
    }
}
