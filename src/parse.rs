use crate::binary::compute_float;
use crate::common::AdjustedMantissa;
use crate::decimal::{parse_decimal, parse_decimal_from_parts};
use crate::float::Float;
use crate::number::{Number, parse_inf_nan, parse_number, parse_number_from_parts};
use crate::simple::parse_long_mantissa;

#[inline]
fn lemire<F: Float>(num: &Number) -> AdjustedMantissa {
    let mut am = compute_float::<F>(num.exponent, num.mantissa);
    if num.many_digits && am != compute_float::<F>(num.exponent, num.mantissa + 1) {
        am.power2 = -1;
    }
    am
}

#[inline]
fn to_float<F: Float>(am: AdjustedMantissa, negative: bool) -> F {
    let mut word = am.mantissa;
    word |= (am.power2 as u64) << F::MANTISSA_EXPLICIT_BITS;
    if negative {
        word |= 1_u64 << F::SIGN_INDEX;
    }
    F::from_u64_bits(word)
}

#[inline]
pub fn parse_from_parts<F: Float>(i: &[u8], f: &[u8], e: i64, negative: bool) -> F {
    if i.is_empty() && f.is_empty() {
        return F::from_u64(0);
    }

    let num = match parse_number_from_parts(i, f, e, negative) {
        Some(n) => n,
        None => return F::from_u64(0),
    };
    if let Some(value) = num.try_fast_path::<F>() {
        return value;
    }

    let mut am = lemire::<F>(&num);
    if am.power2 < 0 {
        am = parse_long_mantissa::<F>(parse_decimal_from_parts(i, f, e, negative));
    }

    to_float(am, num.negative)
}


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

    let mut am = lemire::<F>(&num);
    if am.power2 < 0 {
        am = parse_long_mantissa::<F>(parse_decimal(s));
    }

    let flt = to_float(am, num.negative);
    Some((flt, rest))
}
