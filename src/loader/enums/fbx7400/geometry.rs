//! Geometry-related enums of FBX 7.4 or later.

use std::str::FromStr;

use parser::binary::Result as ParserResult;
use parser::binary::{ParserSource, Attribute, SpecialAttributeType};
use parser::binary::utils::AttributeValue;
use loader::enums::NoSuchVariant;


/// Mapping mode of layer element, hold by `MappingInformationType` node.
///
/// Note that,
///
/// - "a control point" means a vertex (x, y, z),
/// - "a polygon vertex" means an index to control point (in other words, vertex index),
/// - and "a polygon" means group of polygon vertices.
///
/// For detail of those words, see "Class Description" section of [FBX 2018 Developer Help: `FbxMesh`
/// Class Reference](https://help.autodesk.com/view/FBX/2018/ENU/?guid=__cpp_ref_class_fbx_mesh_html).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MappingMode {
    /// The mapping is undetermined.
    None,
    /// One mapping coordinate for each "control point".
    // vertices[i] => map_elem[i]
    ByControlPoint,
    /// One mapping coordinate for each "polygon vertex".
    // vertices[polygon_vertex_index[i]] => map_elem[i]
    ByPolygonVertex,
    /// One mapping coordinate for each "polygon".
    // polygon[i] => map_elem[i]
    ByPolygon,
    /// One mapping coordinate for each "unique edge".
    ///
    /// See [FBX 2018 Developer Help: FbxLayerElement Class
    /// Reference](https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/cpp_ref/class_fbx_layer_element.html#a865a00ff562c164136919bf777abb5e8).
    ByEdge,
    /// Only one mapping coordinate for the whole surface.
    // mesh => map_elem[i]
    AllSame,
}

impl FromStr for MappingMode {
    type Err = NoSuchVariant;

    /// Get a mapping mode value from the property value of a `MappingInformationType` node.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ByControlPoint" | "ByVertex" | "ByVertice" => Ok(MappingMode::ByControlPoint),
            "ByPolygonVertex" => Ok(MappingMode::ByPolygonVertex),
            "ByPolygon" => Ok(MappingMode::ByPolygon),
            "ByEdge" => Ok(MappingMode::ByEdge),
            "AllSame" => Ok(MappingMode::AllSame),
            _ => Err(NoSuchVariant),
        }
    }
}

impl AttributeValue for MappingMode {
    fn from_attribute<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::String {
                return Ok(Self::from_str(&val.into_string()?).ok());
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Self::from_attribute(attr)
    }
}


/// Reference mode of layer element, hold by `ReferenceInformationType` node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceMode {
    /// Direct mapping.
    Direct,
    /// Mapping to index.
    IndexToDirect,
}

impl FromStr for ReferenceMode {
    type Err = NoSuchVariant;

    /// Get a reference mode value from the property value of a `ReferenceInformationType` node.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Direct" => Ok(ReferenceMode::Direct),
            "IndexToDirect" => Ok(ReferenceMode::IndexToDirect),
            _ => Err(NoSuchVariant),
        }
    }
}

impl AttributeValue for ReferenceMode {
    fn from_attribute<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::String {
                return Ok(Self::from_str(&val.into_string()?).ok());
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Self::from_attribute(attr)
    }
}
