use bytemuck::Pod;
use eyre::Result;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use std::any;
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::path::Path;
use std::slice::SliceIndex;

pub static EXE: Exe = Exe {
    inner: OnceCell::new(),
};

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Exe {
    inner: OnceCell<RwLock<Vec<u8>>>,
}

impl Exe {
    /// Internal function to reduce code repetition. `T` is never returned and
    /// is always the return type of the calling function.
    #[inline]
    #[instrument(skip(self))]
    fn __return_out_of_bounds<T>(&self) -> Result<T> {
        error!("Index out of bounds");

        Err(eyre!("Index out of bounds"))
    }

    /// Internal function to reduce code repetition. Returns `self.inner`, or
    /// panics if it was uninitialized.
    #[inline]
    #[instrument(skip(self))]
    fn __inner(&self) -> &RwLock<Vec<u8>> {
        self.inner
            .get()
            .expect("`EXE` was uninitialized, please call `.init(path)`")
    }

    /// Initialize `EXE` with the file at `path`. Will panic if called twice.
    #[inline]
    #[instrument(skip(self))]
    pub fn init(&self, path: impl AsRef<Path> + fmt::Debug) -> Result<()> {
        let inner = fs::read(path.as_ref())?;

        info!("Initializing `EXE`");

        // Check whether the selected file is below 8MB in size, if it is, we print to
        // log. This won't prevent selecting anything other than `SpaceEngine.exe`
        // though it will show what the user did wrong!
        if inner.len() < 8000000usize {
            warn!(
                ?path,
                size = inner.len(),
                "Selected file is below 8MB in size, is this really SpaceEngine?"
            );
        }

        self.inner
            .set(RwLock::new(inner))
            .unwrap_or_else(|_| panic!("`EXE` was already initialized"));

        Ok(())
    }

    /// Get read access.
    #[inline]
    #[instrument(skip(self))]
    pub fn reader(&self) -> RwLockReadGuard<Vec<u8>> {
        self.__inner().read()
    }

    /// Get write access.
    #[inline]
    #[instrument(skip(self))]
    pub fn writer(&self) -> RwLockWriteGuard<Vec<u8>> {
        self.__inner().write()
    }

    /// Try to get read access. Does not block.
    #[inline]
    #[instrument(skip(self))]
    pub fn try_reader(&self) -> Option<RwLockReadGuard<Vec<u8>>> {
        self.__inner().try_read()
    }

    /// Try to get write access. Does not block.
    #[inline]
    #[instrument(skip(self))]
    pub fn try_writer(&self) -> Option<RwLockWriteGuard<Vec<u8>>> {
        self.__inner().try_write()
    }

    /// Get the length of `EXE`.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn len(&self) -> usize {
        self.__inner().read().len()
    }

    /// Get the byte at `index`.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn read(&self, index: usize) -> Option<u8> {
        trace!("Reading byte");

        self.reader().get(index).copied()
    }

    /// Get the bytes in `range`.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn read_many<R>(&self, range: R) -> Option<Vec<u8>>
    where
        R: fmt::Debug + SliceIndex<[u8], Output = [u8]>,
    {
        trace!("Reading bytes");

        self.reader().get(range).map(<[u8]>::to_vec)
    }

    /// Get the bytes at `index` and cast to `P`.
    #[must_use]
    #[inline]
    #[instrument(skip(self), fields(P = any::type_name::<P>()))]
    pub fn read_to<P: Pod>(&self, index: usize) -> Option<P> {
        let range = index..(index + mem::size_of::<P>());

        self.read_many(range).map(|b| *bytemuck::from_bytes(&b))
    }

    /// Read bytes at `index` and cast to a String. Will read until `NULL` is
    /// found or it's read `size` number of bytes. Will panic if it's out of
    /// bounds or invalid utf-8!
    // TODO: This should be refactored, also add some tracing stuff
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn read_to_string(&self, index: usize, size: Option<usize>) -> String {
        let reader = self.reader();
        let mut bytes = vec![];

        for i in 0usize.. {
            let byte = *reader.get(index + i).unwrap();

            if byte == 0u8 || i > size.unwrap_or(usize::MAX) {
                break;
            }

            bytes.push(byte);
        }

        String::from_utf8(bytes).unwrap()
    }

    /// Write the byte in `value` to `index`. Returns the previous bytes, which
    /// can be ignored.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn write(&self, index: usize, value: u8) -> Result<u8> {
        trace!("Writing byte");

        if index > self.len() {
            return self.__return_out_of_bounds::<u8>();
        }

        Ok(mem::replace(&mut self.writer()[index], value))
    }

    /// Write `value` to `index`. Returns the previous bytes, which can
    /// be ignored.
    #[inline]
    #[instrument(skip(self))]
    pub fn write_many(&self, index: usize, value: &[u8]) -> Result<Vec<u8>> {
        trace!("Writing bytes");

        // Range containing every byte of `EXE`
        let range = 0usize..self.len();
        // Range containing every byte of `index` and `value`
        let range_other = index..index + value.len();

        // This should catch all panics possible with `.splice()`
        if !range.contains(&range_other.start) || !range.contains(&range_other.end) {
            return self.__return_out_of_bounds::<Vec<u8>>();
        }

        Ok(self.writer().splice(range_other, value.to_vec()).collect())
    }

    /// Cast `value` to its bytes and write to `index`. Returns the previous
    /// bytes, casted to `P`.
    #[inline]
    #[instrument(skip(self), fields(P = any::type_name::<P>()))]
    pub fn write_to<P: fmt::Debug + Pod>(&self, index: usize, value: P) -> Result<P> {
        self.write_many(index, bytemuck::bytes_of(&value))
            .map(|b| *bytemuck::from_bytes(&b))
    }

    /// Saves the resulting bytes to the file at `path`
    ///
    /// This is only meant to be called once when patching's finished, but
    /// no error will be thrown if it's called multiple times.
    #[inline]
    #[instrument(skip(self))]
    pub fn commit(&self, path: impl AsRef<Path> + fmt::Debug) -> Result<()> {
        info!("Saving `EXE`");

        if env::var("STARB_CALLED_COMMIT").is_ok() {
            warn!("I was called multiple times! This is a bug (albeit benign).")
        }

        // Throw a warning if this is called twice
        env::set_var("STARB_CALLED_COMMIT", "true");

        File::create(path.as_ref())?.write_all(&self.reader())?;

        Ok(())
    }
}
