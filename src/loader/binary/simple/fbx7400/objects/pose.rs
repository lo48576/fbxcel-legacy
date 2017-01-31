//! `Pose` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::arr16_to_mat4x4;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Pose` node with class=`Pose`, subclass=`BindPose`.
///
/// `FbxPose` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Pose {
    /// Object id.
    pub id: i64,
    /// `Type`.
    ///
    /// Always `BindPose`?
    pub type_: String,
    /// `Version`.
    pub version: i32,
    /// Child `PoseNode`s.
    pub pose_info_list: Vec<PoseInfo>,
}

impl Pose {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut type_ = None;
        let mut version = None;
        let mut nb_pose_nodes = None;
        let mut pose_info_list = Vec::new();

        loop {
            let node_type = try_get_node_attrs!(parser, PoseChildAttrs::load);
            match node_type {
                PoseChildAttrs::Type(v) => {
                    type_ = Some(v);
                    parser.skip_current_node()?;
                },
                PoseChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                PoseChildAttrs::NbPoseNodes(v) => {
                    nb_pose_nodes = Some(v);
                    pose_info_list.reserve_exact(v as usize);
                    parser.skip_current_node()?;
                },
                PoseChildAttrs::PoseNode => {
                    pose_info_list.push(PoseInfo::load(parser.subtree_parser())?);
                },
            }
        }
        if ensure_node_exists!(nb_pose_nodes, "NbPoseNodes") as usize != pose_info_list.len() {
            return Err(Error::Inconsistent("Value of `NbPoseNodes` and number of `PoseNode`s \
                                            differ while they should be the same"
                .to_owned()));
        }
        Ok(Pose {
            id: obj_props.id,
            type_: ensure_node_exists!(type_, node_msg!(Pose, Pose, BindPose)),
            version: ensure_node_exists!(version, node_msg!(Pose, Pose, BindPose)),
            pose_info_list: pose_info_list,
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Pose")
            .and_then(|ot| ot.property_template.get("FbxPose"))
    }
}


child_attr_loader! { PoseChildAttrs {
    "Type" => Type(String),
    "Version" => Version(i32),
    "NbPoseNodes" => NbPoseNodes(i32),
    "PoseNode" => PoseNode,
}}


/// `PoseNode` node.
///
/// `FbxPoseInfo` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct PoseInfo {
    /// Matrix.
    ///
    /// `FbxPoseInfo::mMatrixIsLocal`.
    pub matrix: [[f64; 4]; 4],
    /// True if the matrix is defined in local coordinates.
    ///
    /// `FbxPoseInfo::mMatrixIsLocal`.
    ///
    /// Default is `false`?
    pub matrix_is_local: bool,
    /// Node ID.
    ///
    /// `FbxPoseInfo::mNode`.
    pub node: i64,
}

impl PoseInfo {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut matrix = None;
        let mut node = None;

        loop {
            let node_type = try_get_node_attrs!(parser, PoseInfoChildAttrs::load);
            match node_type {
                PoseInfoChildAttrs::Matrix(v) => {
                    matrix = arr16_to_mat4x4(v);
                    if matrix.is_none() {
                        return Err(Error::InvalidAttribute("Matrix".to_owned()));
                    }
                },
                PoseInfoChildAttrs::Node(v) => {
                    node = Some(v);
                },
            }
            parser.skip_current_node()?;
        }
        Ok(PoseInfo {
            matrix: ensure_node_exists!(matrix, "Matrix"),
            // Use `false` as default because `FbxPose::Add()` of FBX SDK 2017.1 uses `false` as
            // default value of `pLocalMatrix` argument.
            matrix_is_local: false,
            node: ensure_node_exists!(node, "Node"),
        })
    }
}


child_attr_loader! { PoseInfoChildAttrs {
    "Matrix" => Matrix(Vec<f64>),
    "Node" => Node(i64),
}}
