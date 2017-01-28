//! `Properties70` node and its children.

use std::fmt;
use fnv::{FnvHashSet, FnvHashMap};
use parser::binary::{Parser, ParserSource, Attributes};
use parser::binary::{Attribute, PrimitiveAttribute};
use loader::binary::simple::{Result, Error};


/// A type of map from property name to value of the specific type.
pub type PropertyMap<T> = FnvHashMap<String, PropertyValue<T>>;


/// Struct to store `Properties70` node data.
#[derive(Default, Clone, PartialEq)]
pub struct Properties70 {
    /// Properties without values.
    values_empty: FnvHashSet<String>,
    /// Values with `i64`, `i32`, or `i16` type.
    values_i64: PropertyMap<i64>,
    /// Values with `f64` or `f32` type.
    values_f64: PropertyMap<f64>,
    /// Values with `[f64; 2]` type.
    values_f64_2: PropertyMap<[f64; 2]>,
    /// Values with `[f64; 3]` type.
    values_f64_3: PropertyMap<[f64; 3]>,
    /// Values with `[f64; 4]` type.
    values_f64_4: PropertyMap<[f64; 4]>,
    /// Values with `[[f64; 4]; 4]` type.
    values_f64_4x4: PropertyMap<[[f64; 4]; 4]>,
    /// Values with `String` type.
    values_string: PropertyMap<String>,
    /// Values with `Vec<u8>` type (called "blob").
    values_binary: PropertyMap<Vec<u8>>,
}

impl Properties70 {
    /// Creates a new `Properties70`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Loads a node from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(parser: P) -> Result<Self> {
        load_properties70(parser)
    }
}

impl fmt::Debug for Properties70 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut format = f.debug_struct("Properties70");
        macro_rules! show {
            ($field:ident) => {
                if !self.$field.is_empty() {
                    format.field(stringify!($field), &self.$field);
                }
            }
        }
        show!(values_empty);
        show!(values_i64);
        show!(values_f64);
        show!(values_f64_2);
        show!(values_f64_3);
        show!(values_f64_4);
        show!(values_f64_4x4);
        show!(values_string);
        show!(values_binary);
        format.finish()
    }
}


/// A type of property value and its metadata.
///
/// Type, label, and flags will be ignored currently.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyValue<T>(T);

impl<T> PropertyValue<T> {
    /// Creates a new `PropertyValue` with the given value.
    pub fn new(v: T) -> Self {
        PropertyValue(v)
    }

    /// Returns the value.
    pub fn value(&self) -> &T {
        &self.0
    }

    /// Returns the value with the ownership.
    pub fn take_value(self) -> T {
        self.0
    }
}

impl<T> From<T> for PropertyValue<T> {
    fn from(v: T) -> Self {
        PropertyValue::new(v)
    }
}


/// Loads a `Properties70` node.
fn load_properties70<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Properties70> {
    let mut props = Properties70::new();

    loop {
        try_get_node_attrs!(parser, |name: &str, attrs| if name == "P" {
            load_property(&mut props, attrs)
        } else {
            warn!("Expected `P` node but got `{}` in `Properties70`", name);
            Err(Error::UnexpectedNode(name.to_owned()))
        });
        parser.skip_current_node()?;
    }
    Ok(props)
}


/// Loads a `P` node in `Properties70`.
fn load_property<R: ParserSource>(
    props: &mut Properties70,
    mut attrs: Attributes<R>
) -> Result<()> {
    use parser::binary::utils::AttributeValues;

    // `type_name`, `label`, `flags` are `String`s, but ignore here because they are currently
    // unused.
    let (name, _type_name, _label, _flags) =
        <(String, (), (), ())>::from_attributes(&mut attrs)?
            .ok_or_else(|| Error::InvalidAttribute("P".to_owned()))?;

    if attrs.rest_attributes() == 0 {
        // Empty attribute.
        props.values_empty.insert(name);
        return Ok(());
    }

    // From multiple attributes, only `[f64]` property can be created.
    // Actually, there are property nodes with name=`filmboxTypeID` seems to have values
    // `5i16, 5i16, 5i16`. However, it seems that it can be regarded as single `5i16`.
    let first: f64 = match attrs.next_attribute()? {
        None => {
            props.values_empty.insert(name);
            return Ok(());
        },
        Some(Attribute::Primitive(PrimitiveAttribute::I16(val))) => {
            props.values_i64.insert(name, (val as i64).into());
            return Ok(());
        },
        Some(Attribute::Primitive(PrimitiveAttribute::I32(val))) => {
            props.values_i64.insert(name, (val as i64).into());
            return Ok(());
        },
        Some(Attribute::Primitive(PrimitiveAttribute::I64(val))) => {
            props.values_i64.insert(name, val.into());
            return Ok(());
        },
        Some(Attribute::Primitive(PrimitiveAttribute::F32(val))) => {
            props.values_f64.insert(name, (val as f64).into());
            return Ok(());
        },
        Some(Attribute::Primitive(PrimitiveAttribute::F64(val))) => val,
        Some(Attribute::Special(attr)) => {
            use parser::binary::SpecialAttributeType;

            let value_type = attr.value_type();
            let vec = attr.into_vec()?;
            match value_type {
                SpecialAttributeType::String => {
                    match String::from_utf8(vec) {
                        Ok(val) => {
                            props.values_string.insert(name, val.into());
                        },
                        Err(err) => {
                            props.values_binary.insert(name, err.into_bytes().into());
                        },
                    }
                },
                SpecialAttributeType::Binary => {
                    props.values_binary.insert(name, vec.into());
                },
            }
            return Ok(());
        },
        _ => return Err(Error::InvalidAttribute("P".into())),
    };

    load_property_rest_f64s(props, attrs, name, first)
}


/// Loads rest `f64` values of a `P` node in `Properties70`.
fn load_property_rest_f64s<R: ParserSource>(
    props: &mut Properties70,
    mut attrs: Attributes<R>,
    name: String,
    first: f64
) -> Result<()> {
    let invalid_attr = || Error::InvalidAttribute("P".into());

    match attrs.rest_attributes() {
        0 => {
            props.values_f64.insert(name, first.into());
            Ok(())
        },
        1 => {
            let second = attrs.convert_into()?.ok_or_else(invalid_attr)?;
            props.values_f64_2.insert(name, [first, second].into());
            Ok(())
        },
        2 => {
            let second = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            let third = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            props.values_f64_3.insert(name, [first, second, third].into());
            Ok(())
        },
        3 => {
            let second = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            let third = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            let fourth = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            props.values_f64_4.insert(name, [first, second, third, fourth].into());
            Ok(())
        },
        15 => {
            let second = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            let third = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            let fourth = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
            let vec1 = [first, second, third, fourth];
            let vec2 = {
                let t: (f64, f64, f64, f64) = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
                [t.0, t.1, t.2, t.3]
            };
            let vec3 = {
                let t: (f64, f64, f64, f64) = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
                [t.0, t.1, t.2, t.3]
            };
            let vec4 = {
                let t: (f64, f64, f64, f64) = attrs.convert_into()?.ok_or_else(&invalid_attr)?;
                [t.0, t.1, t.2, t.3]
            };
            props.values_f64_4x4.insert(name, [vec1, vec2, vec3, vec4].into());
            Ok(())
        },
        n => {
            error!("Supported length of node property with f64 values are 1, 2, 3, 4 and 16, but \
                    got {}",
                   n + 1);
            Err(Error::InvalidAttribute("P".into()))
        },
    }
}
