//! Small clap value parsers.

use std::{fmt::Display, str::FromStr};

/// Parses any `FromStr` type, rendering its error as text.
pub fn parse_enum<T>(s: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: Display,
{
    s.parse::<T>().map_err(|e| e.to_string())
}
