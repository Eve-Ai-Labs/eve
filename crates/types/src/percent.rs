use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percent(u8);

impl Percent {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn inner(&self) -> u8 {
        self.0
    }
}

impl Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for Percent {
    type Error = eyre::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 100 {
            return Err(eyre::eyre!("Percent must be less than 100"));
        }
        Ok(Self(value))
    }
}

impl From<Percent> for u8 {
    fn from(value: Percent) -> Self {
        value.0
    }
}
