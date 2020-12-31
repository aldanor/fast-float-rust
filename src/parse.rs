use std::mem;

use crate::binary::compute_float_from_exp_mantissa;
use crate::common::ByteSlice;
use crate::float::Float;
use crate::format::FloatFormat;
use crate::number::{parse_inf_nan, parse_number};
use crate::simple::parse_long_mantissa;

#[inline]
pub fn parse_float_fmt<F: Float>(mut s: &[u8], fmt: FloatFormat) -> Option<(F, &[u8])> {
    s = s.skip_spaces();
    if s.is_empty() {
        return None;
    }
    let (num, rest) = match parse_number(s, fmt) {
        Some(r) => r,
        None => return parse_inf_nan(s),
    };
    if let Some(value) = num.try_fast_path::<F>() {
        return Some((value, rest));
    }
    let mut am = if num.mantissa == u64::MAX {
        parse_long_mantissa::<F>(s)
    } else {
        compute_float_from_exp_mantissa::<F>(num.exponent, num.mantissa)
    };
    if am.power2 < 0 {
        am = parse_long_mantissa::<F>(s);
    }
    let mut word = am.mantissa;
    word |= (am.power2 as u64) << F::MANTISSA_EXPLICIT_BITS;
    if num.negative {
        word |= 1u64 << F::SIGN_INDEX;
    }
    let value = unsafe {
        if cfg!(target_endian = "big") && mem::size_of::<F>() == 4 {
            *(&word as *const _ as *const F).add(1)
        } else {
            *(&word as *const _ as *const F)
        }
    };
    Some((value, rest))
}

#[inline]
pub fn parse_float<F: Float>(s: &[u8]) -> Option<(F, &[u8])> {
    parse_float_fmt(s, Default::default())
}
