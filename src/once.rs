//! Wrapper around `OnceCell<T>` to provide implementations for `Serialize` and
//! `Deserialize`, as long as `T` implements `Serialize`.

use std::fmt;

use eyre::Result;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

#[inline]
pub fn serialize<S, T>(value: &OnceCell<T>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: fmt::Debug + Serialize,
{
    value.get().unwrap().serialize(ser)
}

#[inline]
pub fn deserialize<'de, D, T>(de: D) -> Result<OnceCell<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + fmt::Debug,
{
    Ok(OnceCell::with_value(T::deserialize(de)?))
}
