//! `Cluster` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::arr16_to_mat4x4;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Deformer` node with class=`SubDeformer`, subclass=`Cluster`.
///
/// `FbxCluster` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `UserData`.
    pub user_data: (String, String),
    /// `Indexes` and `Weights`.
    ///
    /// Lengths of indexes and weights should be same.
    /// If their lengths differs in source FBX data, loader fails and emits error.
    pub indexes_and_weights: Option<(Vec<i32>, Vec<f64>)>,
    /// `Transform`.
    pub transform: [[f64; 4]; 4],
    /// `TransformLink`.
    pub transform_link: [[f64; 4]; 4],
}

impl Cluster {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut user_data = None;
        let mut indexes = None;
        let mut weights = None;
        let mut transform = None;
        let mut transform_link = None;

        loop {
            let node_type = try_get_node_attrs!(parser, ClusterChildAttrs::load);
            match node_type {
                ClusterChildAttrs::Version(v) => {
                    version = Some(v);
                },
                ClusterChildAttrs::UserData(v) => {
                    user_data = Some(v);
                },
                ClusterChildAttrs::Indexes(v) => {
                    indexes = Some(v);
                },
                ClusterChildAttrs::Weights(v) => {
                    weights = Some(v);
                },
                ClusterChildAttrs::Transform(v) => {
                    let mat = arr16_to_mat4x4(v);
                    if mat.is_none() {
                        return Err(Error::InvalidAttribute("Transform".to_owned()));
                    }
                    transform = mat;
                },
                ClusterChildAttrs::TransformLink(v) => {
                    let mat = arr16_to_mat4x4(v);
                    if mat.is_none() {
                        return Err(Error::InvalidAttribute("TransformLink".to_owned()));
                    }
                    transform_link = mat;
                },
            }
            parser.skip_current_node()?;
        }
        // `indexes` and `weights` should be both `Some` or `None`.
        let indexes_and_weights = match (indexes, weights) {
            (Some(i), Some(w)) => {
                if i.len() != w.len() {
                    return Err(Error::Inconsistent("Lengths differs between `Indexes` and \
                                                    `Weights`"
                        .to_owned()));
                }
                Some((i, w))
            },
            (Some(_), None) => {
                return Err(Error::Inconsistent("`Indexes` found but `Weights` not found"
                    .to_owned()))
            },
            (None, Some(_)) => {
                return Err(Error::Inconsistent("`Indexes` not found but `Weights` found"
                    .to_owned()))
            },
            (None, None) => None,
        };
        Ok(Cluster {
            id: obj_props.id,
            version: ensure_node_exists!(version, node_msg!(Deformer, SubDeformer, Cluster)),
            user_data: ensure_node_exists!(user_data, node_msg!(Deformer, SubDeformer, Cluster)),
            indexes_and_weights: indexes_and_weights,
            transform: ensure_node_exists!(transform, node_msg!(Deformer, SubDeformer, Cluster)),
            transform_link: ensure_node_exists!(transform_link,
                                                node_msg!(Deformer, SubDeformer, Cluster)),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Deformer")
            .and_then(|ot| ot.property_template.get("FbxCluster"))
    }
}


child_attr_loader! { ClusterChildAttrs {
    "Version" => Version(i32),
    "UserData" => UserData((String, String)),
    "Indexes" => Indexes(Vec<i32>),
    "Weights" => Weights(Vec<f64>),
    "Transform" => Transform(Vec<f64>),
    "TransformLink" => TransformLink(Vec<f64>),
}}
