use crate::common::{is_8digits, AsciiStr, ByteSlice};
use crate::float::Float;

const MIN_19DIGIT_INT: u64 = 100_0000_0000_0000_0000;

pub const INT_POW10: [u64; 16] = [
    1,
    10,
    100,
    1000,
    10000,
    100000,
    1000000,
    10000000,
    100000000,
    1000000000,
    10000000000,
    100000000000,
    1000000000000,
    10000000000000,
    100000000000000,
    1000000000000000,
];

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
            && self.exponent <= F::MAX_EXPONENT_DISGUISED_FAST_PATH
            && self.mantissa <= F::MAX_MANTISSA_FAST_PATH
            && !self.many_digits
    }

    #[inline]
    pub fn try_fast_path<F: Float>(&self) -> Option<F> {
        if self.is_fast_path::<F>() {
            let mut value = if self.exponent <= F::MAX_EXPONENT_FAST_PATH {
                // normal fast path
                let value = F::from_u64(self.mantissa);
                if self.exponent < 0 {
                    value / F::pow10_fast_path((-self.exponent) as _)
                } else {
                    value * F::pow10_fast_path(self.exponent as _)
                }
            } else {
                // disguised fast path
                let shift = self.exponent - F::MAX_EXPONENT_FAST_PATH;
                let mantissa = self.mantissa.checked_mul(INT_POW10[shift as usize])?;
                if mantissa > F::MAX_MANTISSA_FAST_PATH {
                    return None;
                }
                F::from_u64(mantissa) * F::pow10_fast_path(F::MAX_EXPONENT_FAST_PATH as _)
            };
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
fn parse_8digits(mut v: u64) -> u64 {
    const MASK: u64 = 0x0000_00FF_0000_00FF;
    const MUL1: u64 = 0x000F_4240_0000_0064;
    const MUL2: u64 = 0x0000_2710_0000_0001;
    v -= 0x3030_3030_3030_3030;
    v = (v * 10) + (v >> 8); // will not overflow, fits in 63 bits
    let v1 = (v & MASK).wrapping_mul(MUL1);
    let v2 = ((v >> 16) & MASK).wrapping_mul(MUL2);
    ((v1.wrapping_add(v2) >> 32) as u32) as u64
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
fn try_parse_8digits(s: &mut AsciiStr<'_>, x: &mut u64) {
    // may cause overflows, to be handled later
    if let Some(v) = s.try_read_u64() {
        if is_8digits(v) {
            *x = x
                .wrapping_mul(1_0000_0000)
                .wrapping_add(parse_8digits(v));
            s.step_by(8);
            if let Some(v) = s.try_read_u64() {
                if is_8digits(v) {
                    *x = x
                        .wrapping_mul(1_0000_0000)
                        .wrapping_add(parse_8digits(v));
                    s.step_by(8);
                }
            }
        }
    }
}

#[inline]
fn parse_scientific(s: &mut AsciiStr<'_>) -> i64 {
    // the first character is 'e'/'E' and scientific mode is enabled
    let start = *s;
    s.step();
    let mut exp_num = 0_i64;
    let mut neg_exp = false;
    if !s.is_empty() && s.first_either(b'-', b'+') {
        neg_exp = s.first_is(b'-');
        s.step();
    }
    if s.check_first_digit() {
        s.parse_digits(|digit| {
            if exp_num < 0x10000 {
                exp_num = 10 * exp_num + digit as i64; // no overflows here
            }
        });
        if neg_exp {
            -exp_num
        } else {
            exp_num
        }
    } else {
        *s = start; // ignore 'e' and return back
        0
    }
}

#[inline]
pub fn parse_number(s: &[u8]) -> Option<(Number, usize)> {
    debug_assert!(!s.is_empty());

    let mut s = AsciiStr::new(s);
    let start = s;

    // handle optional +/- sign
    let mut negative = false;
    if s.first() == b'-' {
        negative = true;
        if s.step().is_empty() {
            return None;
        }
    } else if s.first() == b'+' && s.step().is_empty() {
        return None;
    }
    debug_assert!(!s.is_empty());

    // parse initial digits before dot
    let mut mantissa = 0_u64;
    let digits_start = s;
    try_parse_digits(&mut s, &mut mantissa);
    let mut n_digits = s.offset_from(&digits_start);

    // handle dot with the following digits
    let mut n_after_dot = 0;
    let mut exponent = 0_i64;
    let int_end = s;
    if s.check_first(b'.') {
        s.step();
        let before = s;
        try_parse_8digits(&mut s, &mut mantissa);
        try_parse_digits(&mut s, &mut mantissa);
        n_after_dot = s.offset_from(&before);
        exponent = -n_after_dot as i64;
    }

    n_digits += n_after_dot;
    if n_digits == 0 {
        return None;
    }

    // handle scientific format
    let mut exp_number = 0_i64;
    if s.check_first_either(b'e', b'E') {
        exp_number = parse_scientific(&mut s);
        exponent += exp_number;
    }

    let len = s.offset_from(&start) as _;

    // handle uncommon case with many digits
    if n_digits <= 19 {
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

    n_digits -= 19;
    let mut many_digits = false;
    let mut p = digits_start;
    while p.check_first_either(b'0', b'.') {
        n_digits -= p.first().saturating_sub(b'0' - 1) as isize; // '0' = b'.' + 2
        p.step();
    }
    if n_digits > 0 {
        // at this point we have more than 19 significant digits, let's try again
        many_digits = true;
        mantissa = 0;
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
