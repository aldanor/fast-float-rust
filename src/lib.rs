#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::missing_const_for_fn,
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::cargo_common_metadata
)]

use std::error::Error as StdError;
use std::fmt::{self, Display};

mod binary;
mod common;
mod decimal;
mod float;
mod number;
mod parse;
mod simple;

/// Opaque error type for fast-float parsing functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error while parsing a float")
    }
}

impl StdError for Error {}

/// Result type alias for fast-float parsing functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Trait for numerical float types that can be parsed from string.
pub trait FastFloat: float::Float {
    /// Parse a float number from string (full).
    ///
    /// This method parses the float number from the entire string.
    ///
    /// # Errors
    ///
    /// Will return an error either if the string doesn't start with a valid float number,
    /// or if any characterse are left remaining unparsed.
    #[inline]
    fn parse_float<S: AsRef<[u8]>>(s: S) -> Result<Self> {
        let s = s.as_ref();
        match Self::parse_float_partial(s) {
            Ok((v, n)) if n == s.len() => Ok(v),
            _ => Err(Error),
        }
    }

    /// Parse a float number from string (partial).
    ///
    /// This method parses as many digits possible and returns the resulting number along
    /// with the number of digits processed (in case of success, this number is always
    /// positive).
    ///
    /// # Errors
    ///
    /// Will return an error either if the string doesn't start with a valid float number
    /// – that is, if no zero digits were processed.
    #[inline]
    fn parse_float_partial<S: AsRef<[u8]>>(s: S) -> Result<(Self, usize)> {
        parse::parse_float(s.as_ref()).ok_or(Error)
    }
}

impl FastFloat for f32 {}
impl FastFloat for f64 {}

/// Parse a float number from string (full).
///
/// This function parses the float number from the entire string.
///
/// # Errors
///
/// Will return an error either if the string doesn't start with a valid float number,
/// or if any characterse are left remaining unparsed.
#[inline]
pub fn parse<T: FastFloat, S: AsRef<[u8]>>(s: S) -> Result<T> {
    T::parse_float(s)
}

/// Parse a float number from string (partial).
///
/// This function parses as many digits possible and returns the resulting number along
/// with the number of digits processed (in case of success, this number is always
/// positive).
///
/// # Errors
///
/// Will return an error either if the string doesn't start with a valid float number
/// – that is, if no zero digits were processed.
#[inline]
pub fn parse_partial<T: FastFloat, S: AsRef<[u8]>>(s: S) -> Result<(T, usize)> {
    T::parse_float_partial(s)
}
