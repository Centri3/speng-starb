pub mod headers;

use self::headers::PeHeaders;
use bytemuck::Pod;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::mem;
use std::path::Path;
use std::slice::SliceIndex;

pub static EXE: Exe = Exe {
    inner: OnceCell::new(),
};

#[derive(Debug, Default)]
pub struct Exe {
    inner: OnceCell<RwLock<Vec<u8>>>,
}

impl Exe {
    #[inline(always)]
    #[instrument]
    fn inner(&self) -> &RwLock<Vec<u8>> {
        self.inner
            .get()
            .expect("`EXE` was uninitialized, please call `.init(path)`")
    }

    #[inline]
    #[instrument]
    pub fn init(&self, path: impl AsRef<Path> + fmt::Debug) -> Result<(), io::Error> {
        let inner = fs::read(path.as_ref())?;

        // Check whether the selected file is below 8MB in size, if it is, we print to
        // log. This won't prevent selecting anything other than `SpaceEngine.exe`
        // though it will show what the user did wrong!
        if inner.len() < 80000000usize {
            warn!(
                ?path,
                size = inner.len(),
                "Selected file is below 8MB in size"
            )
        }

        self.inner
            .set(RwLock::new(inner))
            // We want to use `.unwrap_or_else` here to prevent printing ~8MB of data, which would
            // probably overflow the stack (it'll still close starb of course, but it's ugly!)
            .unwrap_or_else(|_| panic!("`EXE` was already initialized"));

        Ok(())
    }

    #[inline]
    #[instrument]
    pub fn get_read(&self) -> RwLockReadGuard<Vec<u8>> {
        self.inner().read()
    }

    #[inline]
    #[instrument]
    pub fn get_write(&self) -> RwLockWriteGuard<Vec<u8>> {
        self.inner().write()
    }

    #[must_use]
    #[inline]
    #[instrument]
    pub fn len(&self) -> usize {
        self.inner().read().len()
    }

    #[must_use]
    #[inline]
    #[instrument]
    pub fn read(&self, index: usize) -> Option<u8> {
        self.get_read().get(index).copied()
    }

    #[must_use]
    #[inline]
    #[instrument]
    pub fn read_many<R>(&self, range: R) -> Option<Vec<u8>>
    where
        R: fmt::Debug + SliceIndex<[u8], Output = [u8]>,
    {
        self.get_read().get(range).map(|s| s.to_vec())
    }

    #[must_use]
    #[inline]
    #[instrument]
    pub fn read_to<P: Pod>(&self, index: usize) -> Option<P> {
        self.get_read()
            .get(index..index + mem::size_of::<P>())
            .map(|s| *bytemuck::from_bytes(s))
    }

    #[must_use]
    #[inline]
    #[instrument]
    pub fn write(&self, index: usize, value: u8) {
        self.get_write()[index] = value
    }

    // TODO: I should refactor this, it's ugly
    #[inline]
    #[instrument]
    pub fn write_many<P: fmt::Debug + Pod>(&self, index: usize, value: P) {
        let mut writer = self.get_write();
        let bytes = bytemuck::bytes_of(&value);

        for byte in bytes.iter().enumerate() {
            // This will panic if it's out of bounds!
            writer[index + byte.0] = *byte.1;
        }
    }

    #[inline]
    #[instrument]
    pub fn write_to<P: fmt::Debug + Pod>(&self, index: usize, value: P) {
        let mut writer = self.get_write();

        for (i, byte) in bytemuck::bytes_of(&value).iter().enumerate() {
            writer[i] = *byte;
        }
    }

    #[inline]
    #[instrument]
    pub fn save(&self, path: impl AsRef<Path> + fmt::Debug) -> Result<(), io::Error> {
        File::create(path.as_ref())?.write_all(&self.get_read())?;

        Ok(())
    }

    #[must_use]
    #[inline]
    #[instrument]
    pub fn headers(&self) -> PeHeaders {
        // Offsets from 0x170 where this data lies. Easier to read when given names
        const ENTRY_POINT: usize = 0x28usize;
        const SECTIONS: usize = 0x108usize;

        todo!();
    }
}
