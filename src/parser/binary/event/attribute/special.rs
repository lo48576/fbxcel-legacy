//! Special type node attribute.

use std::io::Read;


/// Special type attribute.
// FIXME: unimplemented.
#[derive(Debug)]
pub struct SpecialAttribute<'a, R: 'a + Read>(&'a mut R);
