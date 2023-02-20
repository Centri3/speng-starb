//! Module for analyzing and patching `SpaceEngine.exe`

pub mod directory;
pub mod headers;

use crate::utils::__report_unreachable;

use self::headers::NtImage;
use self::headers::HEADERS;
use bytemuck::Pod;
use eyre::Report;
use eyre::Result;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use std::any;
use std::env;
use std::ffi::CString;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::path::Path;
use std::slice::SliceIndex;

/// Global variable for `Exe`.
pub static EXE: Exe = Exe::__define();

/// Abstraction over iterating a `Vec<u8>`. Should only be initialized once.
#[derive(Debug)]
#[repr(transparent)]
pub struct Exe {
    inner: OnceCell<RwLock<Vec<u8>>>,
}

impl Exe {
    /// Internal function to define `EXE`
    #[inline(always)]
    const fn __define() -> Self {
        Self {
            inner: OnceCell::new(),
        }
    }

    /// Internal function to reduce code repetition.
    #[inline(always)]
    #[instrument(level = "trace")]
    fn __report_overflow() -> Report {
        eyre!("Overflowed index")
    }

    /// Internal function to reduce code repetition.
    #[inline(always)]
    #[instrument(level = "trace")]
    fn __report_out_of_bounds() -> Report {
        eyre!("Index out of bounds")
    }

    /// Internal function to reduce code repetition. Returns
    /// `.__report_out_of_bounds()` wrapped in `Err`. Cleaner in some cases.
    #[inline(always)]
    #[instrument(level = "trace")]
    fn __return_out_of_bounds() -> Result<()> {
        Err(Self::__report_out_of_bounds())
    }

    /// Internal function to reduce code repetition. Returns `self.inner`
    #[inline(always)]
    #[instrument(skip(self), level = "trace")]
    fn __inner(&self) -> Result<&RwLock<Vec<u8>>> {
        self.inner
            .get()
            .ok_or_else(|| eyre!("`EXE` was uninitialized, please call `.init(path)`"))
    }

    /// Initialize `EXE` with the file at `path`. Also initializes `HEADERS`.
    /// Don't call this twice (at least successfully).
    #[inline(always)]
    #[instrument(skip(self))]
    pub fn init(&self, path: impl AsRef<Path> + fmt::Debug) -> Result<()> {
        let inner = fs::read(path.as_ref())?;

        info!("Initializing `EXE`");

        // Check whether the selected file is below 8MB in size, if it is, we print to
        // log. This won't prevent selecting anything other than `SpaceEngine.exe`
        // though it will show what the user did wrong!
        if inner.len() < 8000000usize {
            warn!(
                size = inner.len(),
                "Selected file is below 8MB in size, is this really SpaceEngine?"
            );
        }

        self.inner
            .set(RwLock::new(inner))
            .map_err(|_| eyre!("`EXE` was already initialized"))?;

        trace!("Initialized `EXE.inner`");

        // Implicitly initialize `HEADERS` if `EXE.init()` is called
        // HEADERS.init()?;

        Ok(())
    }

    /// Get read access. Does not fail, though can dead-lock.
    #[inline]
    #[instrument(skip(self))]
    pub fn reader(&self) -> Result<RwLockReadGuard<Vec<u8>>> {
        Ok(self.__inner()?.read())
    }

    /// Get write access. Does not fail, though can dead-lock.
    #[inline]
    #[instrument(skip(self))]
    pub fn writer(&self) -> Result<RwLockWriteGuard<Vec<u8>>> {
        Ok(self.__inner()?.write())
    }

    /// Try to get exclusive read access. Does not block.
    #[inline]
    #[instrument(skip(self))]
    pub fn try_reader(&self) -> Result<RwLockReadGuard<Vec<u8>>> {
        self.__inner()?
            .try_read()
            .ok_or_else(|| eyre!("Could not get exclusive read access for `EXE`"))
    }

    /// Try to get exclusive write access. Does not block.
    #[inline]
    #[instrument(skip(self))]
    pub fn try_writer(&self) -> Result<RwLockWriteGuard<Vec<u8>>> {
        self.__inner()?
            .try_write()
            .ok_or_else(|| eyre!("Could not get exclusive write access for `EXE`"))
    }

    /// Get the length of `EXE`.
    #[must_use]
    #[inline]
    #[instrument(skip(self))]
    pub fn len(&self) -> Result<usize> {
        Ok(self.__inner()?.read().len())
    }

    /// Get the byte at `index`.
    #[inline]
    #[instrument(skip(self))]
    pub fn read(&self, index: usize) -> Result<u8> {
        trace!("Reading byte");

        self.reader()?
            .get(index)
            .copied()
            .ok_or_else(Self::__report_out_of_bounds)
    }

    /// Get the bytes in `range`.
    #[inline]
    #[instrument(skip(self), fields(R = any::type_name::<R>()))]
    pub fn read_many<R>(&self, range: R) -> Result<Vec<u8>>
    where
        R: fmt::Debug + SliceIndex<[u8], Output = [u8]>,
    {
        trace!("Reading bytes");

        self.reader()?
            .get(range)
            .map(<[u8]>::to_vec)
            .ok_or_else(Self::__report_out_of_bounds)
    }

    /// Get the bytes at `index` and cast to `P`.
    #[inline]
    #[instrument(skip(self), fields(P = any::type_name::<P>()))]
    pub fn read_to<P: Pod>(&self, index: usize) -> Result<P> {
        trace!("Reading to");

        let range = index
            ..(index
                .checked_add(mem::size_of::<P>())
                .ok_or_else(Self::__report_overflow)?);

        self.read_many(range).map(|b| *bytemuck::from_bytes(&b))
    }

    /// Read bytes at `index` and cast to a String. Will read until `NULL` is
    /// found or it's read `size` number of bytes. Will return `Err` if it's out
    /// of bounds or invalid UTF-8! Will also return `Err` if it has `NULL`
    /// outside of trailing `NULL` bytes. Don't read UTF-16.
    ///
    /// You should only specify a size (`Some`) if its size is known, otherwise
    /// you should use `None`.
    #[inline]
    #[instrument(skip(self))]
    pub fn read_to_string(&self, index: usize, size: Option<usize>) -> Result<String> {
        trace!("Reading string");

        // TODO: I'm not sure why clippy recommends this, should this be used here over match?
        let bytes = size.map_or_else(
            || self.__read_to_string_none(index),
            |size| self.__read_to_string_some(index, size),
        )?;

        if !bytes.is_ascii() {
            warn!("String is not ASCII! This may be intentional.");
        }

        // We use CString here to return `Err` if it has `NULL`.
        Ok(CString::new(bytes.as_slice())?.to_str()?.to_string())
    }

    /// Extracted from `.read_to_string()`
    #[inline(always)]
    #[instrument(skip(self), level = "trace")]
    fn __read_to_string_some(&self, index: usize, size: usize) -> Result<Vec<u8>> {
        let bytes = self.read_many(
            index
                ..index
                    .checked_add(size)
                    .ok_or_else(Self::__report_overflow)?,
        )?;

        // Number of `NULL` bytes at the end of `bytes`
        let num_of_nulls = bytes
            .rsplit(|&b| b != 0u8)
            .next()
            .ok_or_else(__report_unreachable)?
            .len();

        Ok(bytes[..bytes.len() - num_of_nulls].to_vec())
    }

    /// Extracted from `.read_to_string()`
    #[inline(always)]
    #[instrument(skip(self), level = "trace")]
    fn __read_to_string_none(&self, index: usize) -> Result<Vec<u8>> {
        // This is quite slow, as every call to `.read_many()` has to create a
        // `Vec<u8>`. It's not a big loss, though, only ~3ms here
        Ok(self
            .read_many(index..)?
            .split(|&b| b == 0u8)
            .next()
            .ok_or_else(__report_unreachable)?
            .to_vec())
    }

    /// Write the byte in `value` to `index`. Returns the previous byte, which
    /// can be ignored.
    #[inline]
    #[instrument(skip(self))]
    pub fn write(&self, index: usize, value: u8) -> Result<u8> {
        trace!("Writing byte");

        if index > self.len()? {
            Self::__return_out_of_bounds()?;
        }

        Ok(mem::replace(&mut self.writer()?[index], value))
    }

    /// Write `value` to `index`. Returns the previous bytes, which can
    /// be ignored.
    #[inline]
    #[instrument(skip(self))]
    pub fn write_many(&self, index: usize, value: &[u8]) -> Result<Vec<u8>> {
        trace!("Writing bytes");

        // Range containing every byte of `EXE`
        let range = 0usize..self.len()?;
        // Range containing every byte of `index` and `value`
        let range_other = index
            ..index
                .checked_add(value.len())
                .ok_or_else(Self::__report_overflow)?;

        // This should catch all panics possible with `.splice()`
        // TODO: Is there a cleaner way to do this?
        if !range.contains(&range_other.start) || !range.contains(&range_other.end) {
            Self::__return_out_of_bounds()?;
        }

        Ok(self.writer()?.splice(range_other, value.to_vec()).collect())
    }

    /// Cast `value` to its bytes and write to `index`. Returns the previous
    /// bytes, casted to `P`.
    #[inline]
    #[instrument(skip(self), fields(P = any::type_name::<P>()))]
    pub fn write_to<P: fmt::Debug + Pod>(&self, index: usize, value: P) -> Result<P> {
        self.write_many(index, bytemuck::bytes_of(&value))
            .map(|b| *bytemuck::from_bytes(&b))
    }

    /// Get `HEADERS`
    #[inline]
    #[instrument(skip(self))]
    pub fn headers(&self) -> &NtImage {
        &HEADERS
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
            warn!("I was called multiple times! This is a bug (albeit benign).");
        }

        // Throw a warning if this is called twice
        env::set_var("STARB_CALLED_COMMIT", "true");

        File::create(path.as_ref())?.write_all(&self.reader()?)?;

        Ok(())
    }
}
