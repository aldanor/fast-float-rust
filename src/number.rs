use crate::common::{is_8digits_le, AsciiStr, ByteSlice};
use crate::float::Float;
use crate::format::FloatFormat;

const MIN_19DIGIT_INT: u64 = 100_0000_0000_0000_0000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Number {
    pub exponent: i64,
    pub mantissa: u64,
    pub negative: bool,
    pub many_digits: bool,
}

impl Number {
    #[inline]
    fn is_fast_path<F: Float>(&self) -> bool {
        F::MIN_EXPONENT_FAST_PATH <= self.exponent
            && self.exponent <= F::MAX_EXPONENT_FAST_PATH
            && self.mantissa <= F::MAX_MANTISSA_FAST_PATH
            && !self.many_digits
    }

    #[inline]
    pub fn try_fast_path<F: Float>(&self) -> Option<F> {
        if self.is_fast_path::<F>() {
            let mut value = F::from_u64(self.mantissa);
            if self.exponent < 0 {
                value = value / F::pow10_fast_path((-self.exponent) as _);
            } else {
                value = value * F::pow10_fast_path(self.exponent as _);
            }
            if self.negative {
                value = -value;
            }
            Some(value)
        } else {
            None
        }
    }
}

#[inline]
fn parse_8digits_le(mut v: u64) -> u64 {
    const MASK: u64 = 0x000000FF000000FF;
    const MUL1: u64 = 0x000F424000000064;
    const MUL2: u64 = 0x0000271000000001;
    v -= 0x3030303030303030;
    v = (v * 10) + (v >> 8); // will not overflow, fits in 63 bits
    let v1 = (v & MASK).wrapping_mul(MUL1);
    let v2 = ((v >> 16) & MASK).wrapping_mul(MUL2);
    let v = (v1.wrapping_add(v2) >> 32) as u32;
    v as u64
}

#[inline]
fn try_parse_digits(s: &mut AsciiStr<'_>, x: &mut u64) {
    s.parse_digits(|digit| {
        *x = x.wrapping_mul(10).wrapping_add(digit as _); // overflows to be handled later
    });
}

#[inline]
fn try_parse_19digits(s: &mut AsciiStr<'_>, x: &mut u64) {
    while *x < MIN_19DIGIT_INT && !s.is_empty() && s.first().is_ascii_digit() {
        let digit = s.first() - b'0';
        *x = (*x * 10) + digit as u64; // no overflows here
        s.step();
    }
}

#[inline]
fn try_parse_8digits_le(s: &mut AsciiStr<'_>, x: &mut u64) -> usize {
    // may cause overflows, to be handled later
    let mut count = 0;
    if cfg!(target_endian = "little") {
        if let Some(v) = s.try_read_u64() {
            if is_8digits_le(v) {
                *x = x
                    .wrapping_mul(1_0000_0000)
                    .wrapping_add(parse_8digits_le(v));
                s.step_by(8);
                count = 8;
                if let Some(v) = s.try_read_u64() {
                    if is_8digits_le(v) {
                        *x = x
                            .wrapping_mul(1_0000_0000)
                            .wrapping_add(parse_8digits_le(v));
                        s.step_by(8);
                        count = 16;
                    }
                }
            }
        }
    }
    count
}

#[inline]
fn parse_scientific(s: &mut AsciiStr<'_>, exponent: &mut i64, fixed: bool) -> Option<()> {
    // the first character is 'e' and scientific mode is enabled
    let start = *s;
    s.step();
    let mut exp_num = 0i64;
    let mut neg_exp = false;
    if !s.is_empty() {
        if s.first_either(b'-', b'+') {
            neg_exp = s.first_is(b'-');
            s.step();
        }
    }
    if s.check_first_digit() {
        s.parse_digits(|digit| {
            if exp_num < 0x10000 {
                exp_num = 10 * exp_num + digit as i64; // no overflows here
            }
        });
        *exponent += if neg_exp { -exp_num } else { exp_num };
    } else if !fixed {
        return None; // error: no integers following 'e'
    } else {
        *s = start; // ignore 'e' and return back
    }
    Some(())
}

#[inline]
pub fn parse_number(s: &[u8], fmt: FloatFormat) -> Option<(Number, usize)> {
    // assuming s.len() >= 1
    let mut s = AsciiStr::new(s);
    let start = s;

    // handle optional +/- sign
    let mut negative = false;
    if s.first_either(b'-', b'+') {
        negative = s.first_is(b'-');
        if s.step().is_empty() {
            return None;
        }
    }

    // parse initial digits before dot
    let mut mantissa = 0u64;
    let digits_start = s;
    try_parse_digits(&mut s, &mut mantissa);
    let mut n_digits = s.offset_from(&digits_start);

    // handle dot with the following digits
    let mut n_after_dot = 0;
    let mut exponent = 0i64;
    let int_end = s;
    if s.check_first(b'.') {
        s.step();
        let before = s;
        try_parse_8digits_le(&mut s, &mut mantissa);
        try_parse_digits(&mut s, &mut mantissa);
        n_after_dot = s.offset_from(&before);
        exponent = -n_after_dot as i64;
    }

    n_digits += n_after_dot;
    if n_digits == 0 {
        return None;
    }

    // handle scientific format
    let mut exp_number = 0i64;
    if fmt.scientific {
        if s.check_first_either(b'e', b'E') {
            parse_scientific(&mut s, &mut exp_number, fmt.fixed)?;
            exponent += exp_number;
        } else if !fmt.fixed {
            return None; // error: scientific and not fixed
        }
    }

    let len = s.offset_from(&start) as _;

    // handle uncommon case with many digits
    n_digits -= 19;
    if n_digits <= 0 {
        return Some((
            Number {
                exponent,
                mantissa,
                negative,
                many_digits: false,
            },
            len,
        ));
    }

    let mut many_digits = false;
    let mut p = digits_start;
    while p.check_first_either(b'0', b'.') {
        n_digits -= p.first().saturating_sub(b'0' - 1) as isize; // '0' = b'.' + 2
        p.step();
    }
    if n_digits > 0 {
        // at this point we have more than 19 significant digits, let's try again
        many_digits = true;
        mantissa = 0u64;
        let mut s = digits_start;
        try_parse_19digits(&mut s, &mut mantissa);
        exponent = if mantissa >= MIN_19DIGIT_INT {
            int_end.offset_from(&s) // big int
        } else {
            s.step(); // fractional component, skip the '.'
            let before = s;
            try_parse_19digits(&mut s, &mut mantissa);
            -s.offset_from(&before)
        } as i64;
        exponent += exp_number; // add back the explicit part
    }

    Some((
        Number {
            exponent,
            mantissa,
            negative,
            many_digits,
        },
        len,
    ))
}

#[inline]
pub fn parse_inf_nan<F: Float>(s: &[u8]) -> Option<(F, usize)> {
    fn parse_inf_rest(s: &[u8]) -> usize {
        if s.len() >= 8 && s[3..].eq_ignore_case(b"inity") {
            8
        } else {
            3
        }
    }
    if s.len() >= 3 {
        if s.eq_ignore_case(b"nan") {
            return Some((F::NAN, 3));
        } else if s.eq_ignore_case(b"inf") {
            return Some((F::INFINITY, parse_inf_rest(s)));
        } else if s.len() >= 4 {
            if s.get_first() == b'+' {
                let s = s.advance(1);
                if s.eq_ignore_case(b"nan") {
                    return Some((F::NAN, 4));
                } else if s.eq_ignore_case(b"inf") {
                    return Some((F::INFINITY, 1 + parse_inf_rest(s)));
                }
            } else if s.get_first() == b'-' {
                let s = s.advance(1);
                if s.eq_ignore_case(b"nan") {
                    return Some((F::NEG_NAN, 4));
                } else if s.eq_ignore_case(b"inf") {
                    return Some((F::NEG_INFINITY, 1 + parse_inf_rest(s)));
                }
            }
        }
    }
    None
}
