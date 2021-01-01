use std::str::FromStr;

use anyhow::{bail, Error, Result};

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
}
