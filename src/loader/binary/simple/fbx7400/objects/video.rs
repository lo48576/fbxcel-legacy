//! `Video` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Video` node with class=`Video`, subclass=`Clip`.
///
/// `FbxVideo` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    /// Object id.
    pub id: i64,
    /// `Type`.
    ///
    /// Always `Clip`?
    pub type_: String,
    /// `UseMipMap`.
    pub use_mip_map: i32,
    /// `Filename`.
    ///
    /// `n` is lower letter.
    pub filename: String,
    /// `RelativeFilename`.
    ///
    /// `n` is lower letter.
    pub relative_filename: String,
    /// `Content`.
    pub content: Vec<u8>,
    /// `Properties70`.
    pub properties: Properties70,
}

impl Video {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut type_ = None;
        let mut use_mip_map = None;
        let mut filename = None;
        let mut relative_filename = None;
        let mut content = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, VideoChildAttrs::load);
            match node_type {
                VideoChildAttrs::Type(v) => {
                    type_ = Some(v);
                    parser.skip_current_node()?;
                },
                VideoChildAttrs::UseMipMap(v) => {
                    use_mip_map = Some(v);
                    parser.skip_current_node()?;
                },
                VideoChildAttrs::Filename(v) => {
                    filename = Some(v);
                    parser.skip_current_node()?;
                },
                VideoChildAttrs::RelativeFilename(v) => {
                    relative_filename = Some(v);
                    parser.skip_current_node()?;
                },
                VideoChildAttrs::Content(v) => {
                    content = Some(v);
                    parser.skip_current_node()?;
                },
                VideoChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Video {
            id: obj_props.id,
            type_: ensure_node_exists!(type_, node_msg!(Video, Video, Clip)),
            use_mip_map: ensure_node_exists!(use_mip_map, node_msg!(Video, Video, Clip)),
            filename: ensure_node_exists!(filename, node_msg!(Video, Video, Clip)),
            relative_filename: ensure_node_exists!(relative_filename,
                                                   node_msg!(Video, Video, Clip)),
            content: ensure_node_exists!(content, node_msg!(Video, Video, Clip)),
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Video")
            .and_then(|ot| ot.property_template.get("FbxVideo"))
    }

    /// `Path`.
    pub fn get_path(&self, defaults: &Option<&Properties70>) -> Option<String> {
        get_property!(self.properties, defaults, values_string, "Path").map(|v| v.value().clone())
    }
}


child_attr_loader! { VideoChildAttrs {
    "Type" => Type(String),
    "UseMipMap" => UseMipMap(i32),
    "Filename" => Filename(String),
    "RelativeFilename" => RelativeFilename(String),
    "Content" => Content(Vec<u8>),
    "Properties70" => Properties70,
}}
