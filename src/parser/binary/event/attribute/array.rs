//! Array type node attribute.

use std::io::Read;


/// Array type attribute.
// FIXME: unimplemented.
#[derive(Debug)]
pub struct ArrayAttribute<'a, R: 'a>(&'a mut R);
