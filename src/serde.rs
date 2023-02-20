//! Contains definitions for remote types, allowing implementations of
//! Serialize/Deserialize where it otherwise isn't implemented.
//!
//! The structure of modules here is the same as it is in the remote crate, so
//! `once_cell::sync::OnceCell` becomes `__once_cell::__sync::__OnceCell`.

use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde::Serialize;

/// Contains definitions for `once_cell`
pub mod __once_cell {
    /// Contains definitions for `once_cell::sync`
    pub mod __sync {
        use super::super::*;

        /// Definition for `once_cell::sync::OnceCell`. `T` must implement
        /// `Serialize`. Will dead-lock if serialized while uninitialized!
        #[derive(Deserialize, Serialize)]
        #[serde(remote = "OnceCell")]
        pub struct __OnceCell<T: Serialize + 'static>(#[serde(getter = "OnceCell::wait")] T);

        impl<T: Serialize + 'static> From<__OnceCell<T>> for OnceCell<T> {
            fn from(value: __OnceCell<T>) -> Self {
                OnceCell::with_value(value.0)
            }
        }
    }
}
