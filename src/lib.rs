//! Excellent FBX loader for Rust programming language.
#![warn(missing_docs)]

#[cfg(feature = "flate2")]
extern crate flate2;
extern crate fnv;
#[cfg(feature = "libflate")]
extern crate libflate;
#[macro_use]
extern crate log;

pub mod loader;
pub mod parser;
