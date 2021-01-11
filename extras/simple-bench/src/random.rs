use std::fmt::{self, Display};
use std::str::FromStr;

use anyhow::{bail, Error, Result};
use fastrand::Rng;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RandomGen {
    Uniform,
    OneOverRand32,
    SimpleUniform32,
    SimpleInt32,
    IntEInt,
    SimpleInt64,
    BigIntDotInt,
    BigInts,
}

impl Display for RandomGen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Uniform => write!(f, "uniform"),
            Self::OneOverRand32 => write!(f, "one_over_rand32"),
            Self::SimpleUniform32 => write!(f, "simple_uniform32"),
            Self::SimpleInt32 => write!(f, "simple_int32"),
            Self::IntEInt => write!(f, "int_e_int"),
            Self::SimpleInt64 => write!(f, "simple_int64"),
            Self::BigIntDotInt => write!(f, "bigint_int_dot_int"),
            Self::BigInts => write!(f, "big_ints"),
        }
    }
}

impl FromStr for RandomGen {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "uniform" => Self::Uniform,
            "one_over_rand32" => Self::OneOverRand32,
            "simple_uniform32" => Self::SimpleUniform32,
            "simple_int32" => Self::SimpleInt32,
            "int_e_int" => Self::IntEInt,
            "simple_int64" => Self::SimpleInt64,
            "bigint_int_dot_int" => Self::BigIntDotInt,
            "big_ints" => Self::BigInts,
            _ => bail!("Invalid random generator: {:?}", s),
        })
    }
}

impl RandomGen {
    pub fn variants() -> &'static [&'static str] {
        &[
            "uniform",
            "one_over_rand32",
            "simple_uniform32",
            "simple_int32",
            "int_e_int",
            "simple_int64",
            "bigint_int_dot_int",
            "big_ints",
        ]
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Uniform,
            Self::OneOverRand32,
            Self::SimpleUniform32,
            Self::SimpleInt32,
            Self::IntEInt,
            Self::SimpleInt64,
            Self::BigIntDotInt,
            Self::BigInts,
        ]
    }

    pub fn gen(&self, rng: &mut Rng) -> String {
        match self {
            Self::Uniform
            | Self::OneOverRand32
            | Self::SimpleUniform32
            | Self::SimpleInt32
            | Self::SimpleInt64 => lexical::to_string(match self {
                Self::Uniform => rng.f64(),
                Self::OneOverRand32 => 1. / rng.u32(1..) as f64,
                Self::SimpleUniform32 => rng.u32(..) as f64 / u32::MAX as f64,
                Self::SimpleInt32 => rng.u32(..) as f64,
                Self::SimpleInt64 => rng.u64(..) as f64,
                _ => unreachable!(),
            }),
            Self::IntEInt => format!("{}e{}", rng.u32(..), rng.u32(..99)),
            Self::BigInts => format!("{}{}{}", rng.u64(..), rng.u64(..), rng.u64(..)),
            Self::BigIntDotInt => format!("{}.{}", rng.u32(..), rng.u32(..)),
        }
    }
}
