use std::error::Error as StdError;
use std::fmt::{self, Display};

mod binary;
mod common;
mod decimal;
mod float;
mod format;
mod number;
mod parse;
mod simple;

pub use format::FloatFormat;

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

pub trait ParseFloat: float::Float {
    #[inline]
    fn parse_float<S: AsRef<[u8]>>(s: S) -> Result<Self> {
        Self::parse_float_fmt(s, FloatFormat::default())
    }

    #[inline]
    fn parse_float_partial<S: AsRef<[u8]>>(s: S) -> Result<(Self, usize)> {
        Self::parse_float_partial_fmt(s, FloatFormat::default())
    }

    #[inline]
    fn parse_float_fmt<S: AsRef<[u8]>>(s: S, fmt: FloatFormat) -> Result<Self> {
        let s = s.as_ref();
        match Self::parse_float_partial_fmt(s, fmt) {
            Ok((v, n)) if n == s.len() => Ok(v),
            _ => Err(Error),
        }
    }

    #[inline]
    fn parse_float_partial_fmt<S: AsRef<[u8]>>(s: S, fmt: FloatFormat) -> Result<(Self, usize)> {
        parse::parse_float_fmt(s.as_ref(), fmt).ok_or(Error)
    }
}

impl ParseFloat for f32 {}
impl ParseFloat for f64 {}

#[inline]
pub fn parse<T: ParseFloat, S: AsRef<[u8]>>(s: S) -> Result<T> {
    T::parse_float(s)
}

#[inline]
pub fn parse_fmt<T: ParseFloat, S: AsRef<[u8]>>(s: S, fmt: FloatFormat) -> Result<T> {
    T::parse_float_fmt(s, fmt)
}

#[inline]
pub fn parse_partial<T: ParseFloat, S: AsRef<[u8]>>(s: S) -> Result<(T, usize)> {
    T::parse_float_partial(s)
}

#[inline]
pub fn parse_partial_fmt<T: ParseFloat, S: AsRef<[u8]>>(
    s: S,
    fmt: FloatFormat,
) -> Result<(T, usize)> {
    T::parse_float_partial_fmt(s, fmt)
}
