use std::ops::BitOr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FloatFormat {
    pub scientific: bool,
    pub fixed: bool,
    pub hex: bool,
}

impl Default for FloatFormat {
    fn default() -> Self {
        Self {
            scientific: true,
            fixed: true,
            hex: false,
        }
    }
}

impl FloatFormat {
    pub const SCIENTIFIC: Self = Self {
        scientific: true,
        fixed: false,
        hex: false,
    };
    pub const FIXED: Self = Self {
        scientific: false,
        fixed: true,
        hex: false,
    };
    pub const HEX: Self = Self {
        scientific: false,
        fixed: false,
        hex: true,
    };
}

impl BitOr for FloatFormat {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            scientific: self.scientific || rhs.scientific,
            fixed: self.fixed || rhs.fixed,
            hex: self.hex || rhs.hex,
        }
    }
}
