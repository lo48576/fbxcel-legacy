//! `Shape` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Geometry` node with class=`Geometry`, subclass=`Shape`.
///
/// `FbxShape` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `Indexes`.
    pub indexes: Vec<i32>,
    /// `Vertices`
    pub vertices: Vec<f64>,
    /// `Normals`
    pub normals: Vec<f64>,
    /// `Properties70`.
    pub properties: Properties70,
}

impl Shape {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut indexes = None;
        let mut vertices = None;
        let mut normals = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, ShapeChildAttrs::load);
            match node_type {
                ShapeChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                ShapeChildAttrs::Indexes(v) => {
                    indexes = Some(v);
                    parser.skip_current_node()?;
                },
                ShapeChildAttrs::Vertices(v) => {
                    vertices = Some(v);
                    parser.skip_current_node()?;
                },
                ShapeChildAttrs::Normals(v) => {
                    normals = Some(v);
                    parser.skip_current_node()?;
                },
                ShapeChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Shape {
            id: obj_props.id,
            version: ensure_node_exists!(version, node_msg!(Geometry, Geometry, Shape)),
            indexes: ensure_node_exists!(indexes, node_msg!(Geometry, Geometry, Shape)),
            vertices: ensure_node_exists!(vertices, node_msg!(Geometry, Geometry, Shape)),
            normals: ensure_node_exists!(normals, node_msg!(Geometry, Geometry, Shape)),
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Geometry")
            .and_then(|ot| ot.property_template.get("FbxShape"))
    }
}


child_attr_loader! { ShapeChildAttrs {
    "Version" => Version(i32),
    "Indexes" => Indexes(Vec<i32>),
    "Vertices" => Vertices(Vec<f64>),
    "Normals" => Normals(Vec<f64>),
    "Properties70" => Properties70,
}}
