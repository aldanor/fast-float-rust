use crate::common::{is_8digits_le, ByteSlice};
use crate::float::Float;
use crate::format::FloatFormat;

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
    v = v.wrapping_mul(10).wrapping_add(v >> 8);
    let v1 = (v & MASK).wrapping_mul(MUL1);
    let v2 = ((v >> 16) & MASK).wrapping_mul(MUL2);
    let v = (v1.wrapping_add(v2) >> 32) as u32;
    v as u64
}

#[inline]
fn try_parse_digits(s: &mut &[u8], i: &mut u64) {
    // may cause overflows, to be handled later
    while !s.is_empty() && s.get_first().is_ascii_digit() {
        // TODO: wrapping sub b'0' first and then check just for < b'9'?
        let digit = s.get_first() - b'0';
        *i = i.wrapping_mul(10).wrapping_add(digit as _);
        *s = s.advance(1);
    }
}

#[inline]
fn try_parse_8digits_le(s: &mut &[u8], i: &mut u64) -> bool {
    // may cause overflows, to be handled later
    if cfg!(target_endian = "little") {
        if s.len() >= 8 {
            let v = s.read_u64();
            if is_8digits_le(v) {
                let digits = parse_8digits_le(v);
                *i = i.wrapping_mul(100000000).wrapping_add(digits);
                *s = s.advance(8);
                return true;
            }
        }
    }
    false
}

#[inline]
fn parse_scientific(s: &mut &[u8], exponent: &mut i64, fixed: bool) -> Option<()> {
    // the first character is 'e' and scientific mode is enabled
    let start = *s;
    let mut exp_num = 0i64;
    *s = s.advance(1);
    let mut neg_exp = false;
    if !s.is_empty() {
        if s.get_first() == b'-' {
            *s = s.advance(1);
            neg_exp = true;
        } else if s.get_first() == b'+' {
            *s = s.advance(1);
        }
    }
    if s.is_empty() || !s.get_first().is_ascii_digit() {
        if !fixed {
            // error: no integers following 'e'
            return None;
        }
        *s = start; // ignore 'e'
    } else {
        while !s.is_empty() && s.get_first().is_ascii_digit() {
            // TODO: wrapping sub b'0' first and then just check for < b'9'?
            let digit = s.get_first() - b'0';
            if exp_num < 0x10000 {
                exp_num = 10 * exp_num + digit as i64; // no overflows here
            }
            *s = s.advance(1);
        }
        *exponent += if neg_exp { -exp_num } else { exp_num };
    }
    Some(())
}

#[inline]
pub fn parse_number(mut s: &[u8], fmt: FloatFormat) -> Option<(Number, &[u8])> {
    // assuming s.len() >= 1
    let c = s.get_first();
    let negative = c == b'-';

    // handle leading +/-
    if c == b'+' || c == b'-' {
        s = s.advance(1);
        if s.is_empty() {
            // error: a single +/-
            return None;
        }
        let c = s.get_first();
        if !c.is_ascii_digit() && c != b'.' {
            // error: +/- must be followed by 0-9 or dot
            return None;
        }
    }

    // parse initial digits
    let mut mantissa = 0u64;
    let start = s;
    try_parse_digits(&mut s, &mut mantissa);

    // handle dot following initial digits
    let mut exponent = 0i64;
    if !s.is_empty() && s.get_first() == b'.' {
        s = s.advance(1);
        let n = s.len();
        if try_parse_8digits_le(&mut s, &mut mantissa) {
            try_parse_8digits_le(&mut s, &mut mantissa);
        }
        try_parse_digits(&mut s, &mut mantissa);
        exponent = -((n - s.len()) as i64);
    }

    if s.len() == start.len() || (s.len() == start.len() - 1 && start.get_first() == b'.') {
        // error: must have encountered at least one digit
        return None;
    }
    let n_digits = start.len() - s.len() - 1;

    // handle scientific format
    if fmt.scientific && !s.is_empty() && (s.get_first() == b'e' || s.get_first() == b'E') {
        parse_scientific(&mut s, &mut exponent, fmt.fixed)?;
    } else if fmt.scientific && !fmt.fixed {
        // error: scientific and not fixed
        return None;
    }

    // handle uncommon case with many digits
    let mut many_digits = false;
    if n_digits >= 19 {
        let mut p = start;
        while !p.is_empty() && (p.get_first() == b'0' || p.get_first() == b'.') {
            p = p.advance(1);
        }
        if n_digits + p.len() >= 19 + start.len() {
            mantissa = 0xFFFFFFFFFFFFFFFF;
            many_digits = true;
        }
    }

    let number = Number {
        exponent,
        mantissa,
        negative,
        many_digits,
    };
    Some((number, s))
}

#[inline]
pub fn parse_inf_nan<F: Float>(s: &[u8]) -> Option<(F, &[u8])> {
    fn parse_inf_rest(s: &[u8]) -> &[u8] {
        if s.len() >= 5 && s.eq_ignore_case(b"inity") {
            &s[5..]
        } else {
            s
        }
    }
    if s.len() >= 3 {
        if s.eq_ignore_case(b"nan") {
            return Some((F::NAN, s.advance(3)));
        } else if s.eq_ignore_case(b"inf") {
            return Some((F::INFINITY, parse_inf_rest(s.advance(3))));
        } else if s.len() >= 4 {
            if s.get_first() == b'+' {
                let s = s.advance(1);
                if s.eq_ignore_case(b"nan") {
                    return Some((F::NAN, s.advance(3)));
                } else if s.eq_ignore_case(b"inf") {
                    return Some((F::INFINITY, parse_inf_rest(s.advance(3))));
                }
            } else if s.get_first() == b'-' {
                let s = s.advance(1);
                if s.eq_ignore_case(b"nan") {
                    return Some((F::NEG_NAN, s.advance(3)));
                } else if s.eq_ignore_case(b"inf") {
                    return Some((F::NEG_INFINITY, parse_inf_rest(s.advance(3))));
                }
            }
        }
    }
    None
}
