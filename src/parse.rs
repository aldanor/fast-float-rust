use core::mem;

use crate::binary::compute_float;
use crate::float::Float;
use crate::number::{parse_inf_nan, parse_number};
use crate::simple::parse_long_mantissa;

#[inline]
pub fn parse_float<F: Float>(s: &[u8]) -> Option<(F, usize)> {
    if s.is_empty() {
        return None;
    }

    let (num, rest) = match parse_number(s) {
        Some(r) => r,
        None => return parse_inf_nan(s),
    };
    if let Some(value) = num.try_fast_path::<F>() {
        return Some((value, rest));
    }

    let mut am = compute_float::<F>(num.exponent, num.mantissa);
    if num.many_digits && am != compute_float::<F>(num.exponent, num.mantissa + 1) {
        am.power2 = -1;
    }
    if am.power2 < 0 {
        am = parse_long_mantissa::<F>(s);
    }

    let mut word = am.mantissa;
    word |= (am.power2 as u64) << F::MANTISSA_EXPLICIT_BITS;
    if num.negative {
        word |= 1_u64 << F::SIGN_INDEX;
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
