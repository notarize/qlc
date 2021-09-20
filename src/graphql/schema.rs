//! Produce consumable schema from introspection JSON
use crate::cli::PrintableMessage;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub mod field;
mod json;

pub type FieldsLookup = HashMap<String, field::Field>;

pub type Documentation = Option<String>;

#[derive(Debug)]
pub enum Error {
    MissingTypeOfForList,
    MissingTypeOfForNonNull,
    MissingNameForField,
    UnknownType { name: String, kind: String },
    FieldsMissingForType(String),
    EnumMissingValues(String),
    InterfaceMissingTypes(String),
    UnionMissingTypes(String),
    JsonParse(serde_json::Error),
}

#[derive(Debug)]
pub struct ObjectType {
    pub fields: FieldsLookup,
}

#[derive(Debug)]
pub struct EnumType {
    pub possible_values: Vec<String>,
}

#[derive(Debug)]
pub struct InterfaceType {
    pub name: String,
    pub fields: FieldsLookup,
    pub possible_types: Vec<String>,
}

#[derive(Debug)]
pub struct InputObjectType {
    pub name: String,
    pub fields: FieldsLookup,
}

#[derive(Debug)]
pub struct UnionType {
    pub name: String,
    pub possible_types: Vec<String>,
    pub fields: FieldsLookup,
}

#[derive(Debug)]
pub enum TypeDefinition {
    Object(ObjectType),
    Enum(EnumType),
    Scalar(String),
    Interface(InterfaceType),
    InputObject(InputObjectType),
    Union(UnionType),
}

impl TypeDefinition {
    pub fn get_fields_lookup(&self) -> Option<&FieldsLookup> {
        match self {
            TypeDefinition::Object(object_type) => Some(&object_type.fields),
            TypeDefinition::InputObject(input_object_type) => Some(&input_object_type.fields),
            TypeDefinition::Interface(interface_type) => Some(&interface_type.fields),
            TypeDefinition::Union(union_type) => Some(&union_type.fields),
            _ => None,
        }
    }
}

fn get_fields_for_complex(
    fields_json: Vec<json::Field>,
    add_typename: bool,
) -> Result<HashMap<String, field::Field>, Error> {
    let mut fields = if add_typename {
        let mut m = HashMap::with_capacity(fields_json.len() + 1);
        m.insert("__typename".to_string(), field::Field::new_type_name());
        m
    } else {
        HashMap::with_capacity(fields_json.len())
    };
    for field_json in fields_json {
        fields.insert(field_json.name.clone(), field::Field::try_from(field_json)?);
    }
    Ok(fields)
}

fn flattened_complex_description(
    descriptions: Option<Vec<json::ComplexObjectDescription>>,
) -> Option<Vec<String>> {
    descriptions.map(|inner| inner.into_iter().map(|v| v.name).collect())
}

#[derive(Debug)]
pub struct Type {
    pub documentation: Documentation,
    pub definition: TypeDefinition,
}

impl TryFrom<json::Type> for Type {
    type Error = Error;

    fn try_from(json: json::Type) -> Result<Self, Self::Error> {
        let json::Type {
            name,
            input_fields,
            fields,
            description,
            kind,
            enum_values,
            possible_types,
        } = json;
        let definition = match kind.as_ref() {
            "OBJECT" => {
                let json_fields = fields.ok_or(Error::FieldsMissingForType(name))?;
                let object_type = ObjectType {
                    fields: get_fields_for_complex(json_fields, true)?,
                };
                TypeDefinition::Object(object_type)
            }
            "ENUM" => {
                let enum_type = EnumType {
                    possible_values: flattened_complex_description(enum_values)
                        .ok_or(Error::EnumMissingValues(name))?,
                };
                TypeDefinition::Enum(enum_type)
            }
            "SCALAR" => TypeDefinition::Scalar(name),
            "INTERFACE" => {
                let json_fields =
                    fields.ok_or_else(|| Error::FieldsMissingForType(name.clone()))?;
                let possible_types = flattened_complex_description(possible_types)
                    .ok_or_else(|| Error::InterfaceMissingTypes(name.clone()))?;
                let interface_type = InterfaceType {
                    name,
                    fields: get_fields_for_complex(json_fields, true)?,
                    possible_types,
                };
                TypeDefinition::Interface(interface_type)
            }
            "INPUT_OBJECT" => {
                let json_fields =
                    input_fields.ok_or_else(|| Error::FieldsMissingForType(name.clone()))?;
                let input_object_type = InputObjectType {
                    name,
                    fields: get_fields_for_complex(json_fields, false)?,
                };
                TypeDefinition::InputObject(input_object_type)
            }
            "UNION" => {
                let possible_types = flattened_complex_description(possible_types)
                    .ok_or_else(|| Error::UnionMissingTypes(name.clone()))?;
                let mut fields = HashMap::with_capacity(1);
                fields.insert("__typename".to_string(), field::Field::new_type_name());
                let union_type = UnionType {
                    name,
                    possible_types,
                    fields,
                };
                TypeDefinition::Union(union_type)
            }
            _ => return Err(Error::UnknownType { name, kind }),
        };
        Ok(Type {
            documentation: description,
            definition,
        })
    }
}

pub struct Schema {
    types: HashMap<String, Type>,
}

impl Schema {
    pub fn try_from_reader(reader: impl Read) -> Result<Self, Error> {
        let schema_json = json::Schema::try_from_reader(reader).map_err(Error::JsonParse)?;
        let mut types = HashMap::with_capacity(schema_json.types.len());
        for type_json in schema_json.types {
            types.insert(type_json.name.clone(), Type::try_from(type_json)?);
        }
        Ok(Schema { types })
    }

    pub fn get_type_for_name(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
}

fn printable_message_error(reason: &str) -> PrintableMessage {
    PrintableMessage::new_simple_compile_error(&format!("malformed schema: {}", reason))
}

pub fn parse_schema(path: &Path) -> Result<Schema, Vec<PrintableMessage>> {
    let file = File::open(path).map_err(|io_error| {
        let error = PrintableMessage::new_compile_error_from_read_io_error(&io_error, path);
        vec![error]
    })?;
    let reader = BufReader::new(file);
    Schema::try_from_reader(reader).map_err(|schema_error| {
        let printable_message = match schema_error {
            Error::MissingTypeOfForNonNull | Error::MissingTypeOfForList => {
                printable_message_error("missing type of information on field")
            }
            Error::MissingNameForField => printable_message_error("missing name on field"),
            Error::UnknownType { name, kind } => printable_message_error(&format!(
                "unknown type defintion `{}` on field `{}`",
                kind, name
            )),
            Error::FieldsMissingForType(name) => {
                printable_message_error(&format!("complex type `{}` is missing fields", name))
            }
            Error::EnumMissingValues(name) => {
                printable_message_error(&format!("enum `{}` is missing variants", name))
            }
            Error::InterfaceMissingTypes(name) => {
                printable_message_error(&format!("interface `{}` has no implementations", name))
            }
            Error::UnionMissingTypes(name) => {
                printable_message_error(&format!("union `{}` has no implementations", name))
            }
            Error::JsonParse(serde_error) => {
                printable_message_error(&format!("JSON parse error: {}", serde_error))
            }
        };
        vec![printable_message]
    })
}
