use std::ptr;

// Most of these are inherently unsafe; we assume we know what we're calling and when.
pub trait ByteSlice: AsRef<[u8]> + AsMut<[u8]> {
    #[inline]
    fn get_at(&self, i: usize) -> u8 {
        unsafe { *self.as_ref().get_unchecked(i) }
    }

    #[inline]
    fn get_first(&self) -> u8 {
        debug_assert!(!self.as_ref().is_empty());
        self.get_at(0)
    }

    #[inline]
    fn check_first(&self, c: u8) -> bool {
        !self.as_ref().is_empty() && self.get_first() == c
    }

    #[inline]
    fn check_first2(&self, c1: u8, c2: u8) -> bool {
        !self.as_ref().is_empty() && (self.get_first() == c1 || self.get_first() == c2)
    }

    #[inline]
    fn eq_ignore_case(&self, u: &[u8]) -> bool {
        debug_assert!(self.as_ref().len() >= u.len());
        let d = (0..u.len()).fold(0, |d, i| d | self.get_at(i) ^ u.get_at(i));
        d == 0 || d == 32
    }

    #[inline]
    fn advance(&self, n: usize) -> &[u8] {
        &self.as_ref()[n..]
    }

    #[inline]
    fn skip_spaces(&self) -> &[u8] {
        let mut s = self.as_ref();
        while !s.is_empty() && is_space(s.get_first()) {
            s = s.advance(1);
        }
        s
    }

    #[inline]
    fn skip_chars(&self, c: u8) -> &[u8] {
        let mut s = self.as_ref();
        while s.check_first(c) {
            s = s.advance(1);
        }
        s
    }

    #[inline]
    fn skip_chars2(&self, c1: u8, c2: u8) -> &[u8] {
        let mut s = self.as_ref();
        while !s.is_empty() && (s.get_first() == c1 || s.get_first() == c2) {
            s = s.advance(1);
        }
        s
    }

    #[inline]
    fn read_u64(&self) -> u64 {
        debug_assert!(self.as_ref().len() >= 8);
        let mut value = 0u64;
        let src = self.as_ref().as_ptr();
        let dst = &mut value as *mut _ as *mut u8;
        unsafe { ptr::copy_nonoverlapping(src, dst, 8) };
        value
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        debug_assert!(self.as_ref().len() >= 8);
        let src = &value as *const _ as *const u8;
        let dst = self.as_mut().as_mut_ptr();
        unsafe { ptr::copy_nonoverlapping(src, dst, 8) };
    }
}

impl ByteSlice for [u8] {}

#[inline]
fn is_space(c: u8) -> bool {
    const TABLE: [bool; 33] = [
        false, false, false, false, false, false, false, false, false, true, true, true, true,
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ];
    (c <= 32) && unsafe { *TABLE.get_unchecked(c as usize) }
}

#[inline]
pub fn is_8digits_le(v: u64) -> bool {
    let a = v.wrapping_add(0x4646464646464646);
    let b = v.wrapping_sub(0x3030303030303030);
    (a | b) & 0x8080808080808080 == 0
}

#[inline]
pub fn parse_digits(s: &mut &[u8], mut f: impl FnMut(u8)) {
    while !s.is_empty() {
        let c = s.get_first().wrapping_sub(b'0');
        if c < 10 {
            f(c);
            *s = s.advance(1);
        } else {
            break;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct AdjustedMantissa {
    pub mantissa: u64,
    pub power2: i32,
}

impl AdjustedMantissa {
    #[inline]
    pub const fn zero_pow2(power2: i32) -> Self {
        Self {
            mantissa: 0,
            power2,
        }
    }
}
