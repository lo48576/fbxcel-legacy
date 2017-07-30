//! Enums of FBX 7.4 or later.

pub use self::deformer::SkinningType;
pub use self::geometry::{MappingMode, ReferenceMode};
pub use self::texture::{BlendMode, WrapMode};

pub mod deformer;
pub mod geometry;
pub mod texture;
