//! FBX enums.

use std::error::Error;
use std::fmt;

pub mod fbx7400;


/// No such variant error.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoSuchVariant;

impl fmt::Display for NoSuchVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No such variant")
    }
}

impl Error for NoSuchVariant {
    fn description(&self) -> &str {
        "No such variant"
    }
}
