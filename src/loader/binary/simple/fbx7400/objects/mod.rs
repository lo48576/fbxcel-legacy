//! `Objects` node and its children.

use fnv::FnvHashMap;
use parser::binary::{Parser, ParserSource, Attributes};
use loader::binary::simple::{Result, GenericNode};
use loader::binary::simple::fbx7400::separate_name_class;


macro_rules! node_msg {
    ($node:ident, $class:ident, $subclass:ident) => {
        concat!(stringify!($node),
                " (class=`",
                stringify!($class),
                "`, subclass=`",
                stringify!($subclass),
                "`)")
    }
}


macro_rules! get_property {
    ($obj_props:expr, $def_props_opt:expr, $field:ident, $name:expr) => {
        $obj_props
            .$field
            .get($name)
            .or_else(|| $def_props_opt.and_then(|d| d.$field.get($name)))
    };
}


pub mod animation_curve_node;
pub mod blend_shape;
pub mod blend_shape_channel;
pub mod cluster;
pub mod display_layer;
pub mod material;
pub mod model;
pub mod null;
pub mod pose;
pub mod shape;
pub mod skeleton;
pub mod skin;
pub mod texture;
pub mod video;


/// Map type with key = `i64`.
pub type ObjectMap<T> = FnvHashMap<i64, T>;


/// Properties common to object nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperties {
    /// ID.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Class.
    pub class: String,
    /// Subclass.
    pub subclass: String,
}

impl ::parser::binary::utils::AttributeValues for ObjectProperties {
    fn from_attributes<R: ParserSource>(attrs: &mut Attributes<R>)
        -> ::parser::binary::Result<Option<Self>> {
        let (id, name_class, subclass) =
            match <(i64, String, String)>::from_attributes(attrs)? {
                Some(v) => v,
                None => return Ok(None),
            };
        Ok(separate_name_class(&name_class).map(|(name, class)| {
            ObjectProperties {
                id: id,
                name: name.to_owned(),
                class: class.to_owned(),
                subclass: subclass,
            }
        }))
    }
}


/// `Objects`.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Objects {
    /// `AnimationCurveNode`.
    pub animation_curve_node: ObjectMap<animation_curve_node::AnimationCurveNode>,
    /// `BlendShape`.
    pub blend_shape: ObjectMap<blend_shape::BlendShape>,
    /// `BlendShapeChannel`.
    pub blend_shape_channel: ObjectMap<blend_shape_channel::BlendShapeChannel>,
    /// `Cluster`.
    pub cluster: ObjectMap<cluster::Cluster>,
    /// `DisplayLayer`.
    pub display_layer: ObjectMap<display_layer::DisplayLayer>,
    /// `Material`.
    pub material: ObjectMap<material::Material>,
    /// `Model` (class=`Model`, subclass=`Camera`).
    pub model_camera: ObjectMap<model::Model>,
    /// `Model` (class=`Model`, subclass=`Light`).
    pub model_light: ObjectMap<model::Model>,
    /// `Model` (class=`Model`, subclass=`LimbNode`).
    pub model_limbnode: ObjectMap<model::Model>,
    /// `Model` (class=`Model`, subclass=`Mesh`).
    pub model_mesh: ObjectMap<model::Model>,
    /// `Model` (class=`Model`, subclass=`Null`).
    pub model_null: ObjectMap<model::Model>,
    /// `Model` (class=`Model`, subclass is unknown).
    pub model_unknown: ObjectMap<model::Model>,
    /// `Null`.
    pub null: ObjectMap<null::Null>,
    /// `Pose`.
    pub pose: ObjectMap<pose::Pose>,
    /// `Shape`.
    pub shape: ObjectMap<shape::Shape>,
    /// `Skeleton`.
    pub skeleton: ObjectMap<skeleton::Skeleton>,
    /// `Skin`.
    pub skin: ObjectMap<skin::Skin>,
    /// `Texture`.
    pub texture: ObjectMap<texture::Texture>,
    /// `Video`.
    pub video: ObjectMap<video::Video>,
    /// Unknown type.
    pub unknown: ObjectMap<UnknownObject>,
}

impl Objects {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut objects: Objects = Default::default();

        loop {
            let obj_props = try_get_node_attrs!(parser, load_object_property);
            let mut unknown_obj = false;
            // Node name is inferable from object class, therefor the code here only requires
            // object properties to decide node type.
            match (obj_props.class.as_str(), obj_props.subclass.as_str()) {
                // `Model`.
                ("Model", subclass) => {
                    let id = obj_props.id;
                    let obj = model::Model::load(parser.subtree_parser(), &obj_props)?;
                    let target = match subclass {
                        "Camera" => &mut objects.model_camera,
                        "Light" => &mut objects.model_light,
                        "LimbNode" => &mut objects.model_limbnode,
                        "Mesh" => &mut objects.model_mesh,
                        "Null" => &mut objects.model_null,
                        _ => {
                            warn!("Unknown model subclass: id={:?}, subclass={:?}",
                                  obj_props.id,
                                  obj_props);
                            &mut objects.model_unknown
                        },
                    };
                    target.insert(id, obj);
                },
                // `NodeAttribute`.
                ("NodeAttribute", "LimbNode") => {
                    let obj = skeleton::Skeleton::load(parser.subtree_parser(), &obj_props)?;
                    objects.skeleton.insert(obj_props.id, obj);
                },
                ("NodeAttribute", "Null") => {
                    let obj = null::Null::load(parser.subtree_parser(), &obj_props)?;
                    objects.null.insert(obj_props.id, obj);
                },
                // `AnimationCurveNode`.
                ("AnimCurveNode", "") => {
                    let obj =
                        animation_curve_node::AnimationCurveNode::load(parser.subtree_parser(),
                                                                       &obj_props)?;
                    objects.animation_curve_node.insert(obj_props.id, obj);
                },
                // `CollectionExclusive`.
                ("DisplayLayer", "DisplayLayer") => {
                    let obj = display_layer::DisplayLayer::load(parser.subtree_parser(),
                                                                &obj_props)?;
                    objects.display_layer.insert(obj_props.id, obj);
                },
                // `Deformer`.
                ("Deformer", "BlendShape") => {
                    let obj = blend_shape::BlendShape::load(parser.subtree_parser(), &obj_props)?;
                    objects.blend_shape.insert(obj_props.id, obj);
                },
                ("Deformer", "Skin") => {
                    let obj = skin::Skin::load(parser.subtree_parser(), &obj_props)?;
                    objects.skin.insert(obj_props.id, obj);
                },
                ("SubDeformer", "BlendShapeChannel") => {
                    let obj =
                        blend_shape_channel::BlendShapeChannel::load(parser.subtree_parser(),
                                                                     &obj_props)?;
                    objects.blend_shape_channel.insert(obj_props.id, obj);
                },
                ("SubDeformer", "Cluster") => {
                    let obj = cluster::Cluster::load(parser.subtree_parser(), &obj_props)?;
                    objects.cluster.insert(obj_props.id, obj);
                },
                // `Geometry`.
                ("Geometry", "Shape") => {
                    let obj = shape::Shape::load(parser.subtree_parser(), &obj_props)?;
                    objects.shape.insert(obj_props.id, obj);
                },
                // `Material`.
                ("Material", "") => {
                    let obj = material::Material::load(parser.subtree_parser(), &obj_props)?;
                    objects.material.insert(obj_props.id, obj);
                },
                // `Pose`.
                ("Pose", "BindPose") => {
                    let obj = pose::Pose::load(parser.subtree_parser(), &obj_props)?;
                    objects.pose.insert(obj_props.id, obj);
                },
                // `Texture`.
                ("Texture", "") => {
                    let obj = texture::Texture::load(parser.subtree_parser(), &obj_props)?;
                    objects.texture.insert(obj_props.id, obj);
                },
                // `Video`.
                ("Video", "Clip") => {
                    let obj = video::Video::load(parser.subtree_parser(), &obj_props)?;
                    objects.video.insert(obj_props.id, obj);
                },
                _ => {
                    // Here `obj_props` is borrowed and it can't be passed for
                    // `UnknownObject::load()`.
                    unknown_obj = true;
                },
            }
            if unknown_obj {
                warn!("Unknown object type: {:?}", obj_props);
                let id = obj_props.id;
                let obj = UnknownObject::load(parser.subtree_parser(), obj_props)?;
                objects.unknown.insert(id, obj);
            }
        }
        Ok(objects)
    }
}


/// Loads `ObjectProperties` in the same manner as usual child node attributes.
fn load_object_property<R: ParserSource>(
    name: &str,
    mut attrs: Attributes<R>
) -> Result<ObjectProperties> {
    use parser::binary::utils::AttributeValues;
    use loader::binary::simple::Error;

    <ObjectProperties>::from_attributes(&mut attrs)
        ?
        .ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
}


/// An object node of unknown type.
#[derive(Debug, Clone, PartialEq)]
pub struct UnknownObject {
    /// Object properties.
    pub properties: ObjectProperties,
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl UnknownObject {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        attrs: ObjectProperties
    ) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(UnknownObject {
            properties: attrs,
            nodes: nodes,
        })
    }
}
