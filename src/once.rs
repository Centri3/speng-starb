//! Wrapper around `OnceCell<T>` to provide implementations for `Serialize` and
//! `Deserialize`, as long as `T` implements `Serialize`.

// We don't care if a function's unused
#![allow(dead_code)]

use once_cell::sync::OnceCell as Imp;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::fmt;

/// A thread-safe cell which can be written to only once.
///
/// `OnceCell` provides `&` references to the contents without RAII guards.
///
/// Reading a non-`None` value out of `OnceCell` establishes a
/// happens-before relationship with a corresponding write. For example, if
/// thread A initializes the cell with `get_or_init(f)`, and thread B
/// subsequently reads the result of this call, B also observes all the side
/// effects of `f`.
///
/// # Example
/// ```
/// use once_cell::sync::OnceCell;
///
/// static CELL: OnceCell<String> = OnceCell::new();
/// assert!(CELL.get().is_none());
///
/// std::thread::spawn(|| {
///     let value: &String = CELL.get_or_init(|| {
///         "Hello, World!".to_string()
///     });
///     assert_eq!(value, "Hello, World!");
/// }).join().unwrap();
///
/// let value: Option<&String> = CELL.get();
/// assert!(value.is_some());
/// assert_eq!(value.unwrap().as_str(), "Hello, World!");
/// ```
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct OnceCell<T: fmt::Debug + Serialize>(Imp<T>);

impl<T: fmt::Debug + Serialize> OnceCell<T> {
    /// Creates a new empty cell.
    #[inline(always)]
    pub const fn new() -> Self {
        Self(Imp::new())
    }

    /// Creates a new initialized cell.
    #[inline(always)]
    pub const fn with_value(value: T) -> Self {
        Self(Imp::with_value(value))
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized. This method
    /// never blocks.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }

    /// Gets the reference to the underlying value, blocking the current thread
    /// until it is set.
    #[inline]
    pub fn wait(&self) -> &T {
        self.0.wait()
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    ///
    /// This method is allowed to violate the invariant of writing to a
    /// `OnceCell` at most once because it requires `&mut` access to `self`.
    /// As with all interior mutability, `&mut` access permits arbitrary
    /// modification:
    ///
    /// ```
    /// use once_cell::sync::OnceCell;
    ///
    /// let mut cell: OnceCell<u32> = OnceCell::new();
    /// cell.set(92).unwrap();
    /// cell = OnceCell::new();
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.0.get_mut()
    }

    /// Get the reference to the underlying value, without checking if the
    /// cell is initialized.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the cell is in initialized state, and that
    /// the contents are acquired by (synchronized to) this thread.
    #[inline]
    pub unsafe fn get_unchecked(&self) -> &T {
        self.0.get_unchecked()
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell was empty and `Err(value)` if it was full.
    #[inline]
    pub fn set(&self, value: T) -> Result<(), T> {
        self.0.set(value)
    }

    /// Like [`set`](Self::set), but also returns a reference to the final cell
    /// value.
    ///
    /// # Example
    ///
    /// ```
    /// use once_cell::unsync::OnceCell;
    ///
    /// let cell = OnceCell::new();
    /// assert!(cell.get().is_none());
    ///
    /// assert_eq!(cell.try_insert(92), Ok(&92));
    /// assert_eq!(cell.try_insert(62), Err((&92, 62)));
    ///
    /// assert!(cell.get().is_some());
    /// ```
    #[inline]
    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        self.0.try_insert(value)
    }

    /// Gets the contents of the cell, initializing it with `f` if the cell
    /// was empty.
    ///
    /// Many threads may call `get_or_init` concurrently with different
    /// initializing functions, but it is guaranteed that only one function
    /// will be executed.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`. The
    /// exact outcome is unspecified. Current implementation deadlocks, but
    /// this may be changed to a panic in the future.
    ///
    /// # Example
    /// ```
    /// use once_cell::sync::OnceCell;
    ///
    /// let cell = OnceCell::new();
    /// let value = cell.get_or_init(|| 92);
    /// assert_eq!(value, &92);
    /// let value = cell.get_or_init(|| unreachable!());
    /// assert_eq!(value, &92);
    /// ```
    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(f)
    }

    /// Gets the contents of the cell, initializing it with `f` if
    /// the cell was empty. If the cell was empty and `f` failed, an
    /// error is returned.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and
    /// the cell remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`.
    /// The exact outcome is unspecified. Current implementation
    /// deadlocks, but this may be changed to a panic in the future.
    ///
    /// # Example
    /// ```
    /// use once_cell::sync::OnceCell;
    ///
    /// let cell = OnceCell::new();
    /// assert_eq!(cell.get_or_try_init(|| Err(())), Err(()));
    /// assert!(cell.get().is_none());
    /// let value = cell.get_or_try_init(|| -> Result<i32, ()> {
    ///     Ok(92)
    /// });
    /// assert_eq!(value, Ok(&92));
    /// assert_eq!(cell.get(), Some(&92))
    /// ```
    #[inline]
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.0.get_or_try_init(f)
    }

    /// Takes the value out of this `OnceCell`, moving it back to an
    /// uninitialized state.
    ///
    /// Has no effect and returns `None` if the `OnceCell` hasn't been
    /// initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use once_cell::sync::OnceCell;
    ///
    /// let mut cell: OnceCell<String> = OnceCell::new();
    /// assert_eq!(cell.take(), None);
    ///
    /// let mut cell = OnceCell::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.take(), Some("hello".to_string()));
    /// assert_eq!(cell.get(), None);
    /// ```
    ///
    /// This method is allowed to violate the invariant of writing to a
    /// `OnceCell` at most once because it requires `&mut` access to `self`.
    /// As with all interior mutability, `&mut` access permits arbitrary
    /// modification:
    ///
    /// ```
    /// use once_cell::sync::OnceCell;
    ///
    /// let mut cell: OnceCell<u32> = OnceCell::new();
    /// cell.set(92).unwrap();
    /// cell = OnceCell::new();
    /// ```
    #[inline]
    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }

    /// Consumes the `OnceCell`, returning the wrapped value. Returns
    /// `None` if the cell was empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use once_cell::sync::OnceCell;
    ///
    /// let cell: OnceCell<String> = OnceCell::new();
    /// assert_eq!(cell.into_inner(), None);
    ///
    /// let cell = OnceCell::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.into_inner(), Some("hello".to_string()));
    /// ```
    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.0.into_inner()
    }
}

impl<T> Serialize for OnceCell<T>
where
    T: fmt::Debug + Serialize,
{
    #[inline]
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.0.get().unwrap().serialize(ser)
    }
}

impl<'de, T> Deserialize<'de> for OnceCell<T>
where
    T: Deserialize<'de> + fmt::Debug + Serialize,
{
    #[inline]
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self(Imp::with_value(T::deserialize(de)?)))
    }
}
