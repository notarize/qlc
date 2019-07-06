//! Produce consumable schmea from introspetion JSON
use json::*;
use std::collections::HashMap;
use std::io::Read;

mod json;

pub type FieldsLookup = HashMap<String, Field>;

#[derive(Debug)]
pub enum Error {
    MissingTypeOfForList,
    MissingTypeOfForNonNull,
    MissingNameForField,
    UnknownTypeKind(String, String),
    EnumMissingValues(String),
    JSONParseError(serde_json::Error),
}

#[derive(Debug)]
pub enum ScalarType {
    Custom(String),
    Boolean,
    String,
    Float,
    Int,
    ID,
}

impl ScalarType {
    fn from_name(name: &str) -> Self {
        match name {
            "Boolean" => ScalarType::Boolean,
            "String" => ScalarType::String,
            "Float" => ScalarType::Float,
            "Int" => ScalarType::Int,
            "ID" => ScalarType::ID,
            _ => ScalarType::Custom(name.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct FieldType {
    pub nullable: bool,
    pub definition: FieldTypeDefintion,
}

#[derive(Debug)]
pub enum FieldTypeDefintion {
    List(Box<FieldType>),
    Object(String),
    Interface(String),
    Enum(String),
    Scalar(ScalarType),
}

impl FieldType {
    fn from_field_type_json(json: FieldSubtypeJSON) -> Result<Self, Error> {
        let mut nullable = true;
        let mut iter = json;
        loop {
            match iter.kind.as_ref() {
                "NON_NULL" => {
                    nullable = false;
                    iter = *iter.of_type.ok_or_else(|| Error::MissingTypeOfForNonNull)?;
                }
                "LIST" => {
                    iter = *iter.of_type.ok_or_else(|| Error::MissingTypeOfForList)?;
                    let field_type = FieldType::from_field_type_json(iter)?;
                    let definition = FieldTypeDefintion::List(Box::new(field_type));
                    return Ok(FieldType {
                        definition,
                        nullable,
                    });
                }
                "OBJECT" => {
                    let name = iter.name.ok_or_else(|| Error::MissingNameForField)?;
                    let definition = FieldTypeDefintion::Object(name);
                    return Ok(FieldType {
                        definition,
                        nullable,
                    });
                }
                "SCALAR" => {
                    let name = iter.name.ok_or_else(|| Error::MissingNameForField)?;
                    let definition = FieldTypeDefintion::Scalar(ScalarType::from_name(&name));
                    return Ok(FieldType {
                        definition,
                        nullable,
                    });
                }
                "INTERFACE" => {
                    let name = iter.name.ok_or_else(|| Error::MissingNameForField)?;
                    let definition = FieldTypeDefintion::Interface(name);
                    return Ok(FieldType {
                        definition,
                        nullable,
                    });
                }
                "ENUM" => {
                    let name = iter.name.ok_or_else(|| Error::MissingNameForField)?;
                    let definition = FieldTypeDefintion::Enum(name);
                    return Ok(FieldType {
                        definition,
                        nullable,
                    });
                }
                _ => {
                    // TODO
                    let definition = FieldTypeDefintion::Scalar(ScalarType::Boolean);
                    return Ok(FieldType {
                        definition,
                        nullable,
                    });
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub type_description: FieldType,
}

impl Field {
    fn from_field_json(json: FieldJSON) -> Self {
        let type_desc = json.type_desc;
        Field {
            name: json.name,
            description: json.description,
            type_description: FieldType::from_field_type_json(type_desc).unwrap(),
        }
    }
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
}

#[derive(Debug)]
pub enum TypeDefintion {
    Object(ObjectType),
    Enum(EnumType),
    Scalar(String),
    Interface(InterfaceType),
}

#[derive(Debug)]
pub struct Type {
    pub description: Option<String>,
    pub definition: TypeDefintion,
}

impl Type {
    fn from_type_json(json: TypeJSON) -> Result<Self, Error> {
        let definition = match json.kind.as_ref() {
            "OBJECT" => {
                let fields_json = json.fields.unwrap_or_else(Vec::new);
                let mut fields = HashMap::with_capacity(fields_json.len());
                for field_json in fields_json {
                    fields.insert(field_json.name.clone(), Field::from_field_json(field_json));
                }
                let object_type = ObjectType { fields };
                TypeDefintion::Object(object_type)
            }
            "ENUM" => {
                let name = &json.name;
                let possible_values = json
                    .enum_values
                    .ok_or_else(|| Error::EnumMissingValues(name.clone()))?
                    .iter()
                    .map(|v| v.name.clone())
                    .collect();
                let enum_type = EnumType { possible_values };
                TypeDefintion::Enum(enum_type)
            }
            "SCALAR" => TypeDefintion::Scalar(json.name),
            "INTERFACE" => {
                let fields_json = json.fields.unwrap_or_else(Vec::new);
                let mut fields = HashMap::with_capacity(fields_json.len());
                for field_json in fields_json {
                    fields.insert(field_json.name.clone(), Field::from_field_json(field_json));
                }
                let interface_type = InterfaceType {
                    name: json.name,
                    fields,
                };
                TypeDefintion::Interface(interface_type)
            }
            // TODO
            "INPUT_OBJECT" | "UNION" => TypeDefintion::Scalar(json.name),
            _ => return Err(Error::UnknownTypeKind(json.name, json.kind)),
        };
        Ok(Type {
            description: json.description,
            definition,
        })
    }
}

pub struct Schema {
    types: HashMap<String, Type>,
}

impl Schema {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let schema_json = SchemaJSON::from_reader(reader).map_err(Error::JSONParseError)?;
        let mut types = HashMap::with_capacity(schema_json.types.len());
        for type_json in schema_json.types {
            let name = type_json.name.clone();
            let processed_type = Type::from_type_json(type_json)?;
            types.insert(name, processed_type);
        }
        Ok(Schema { types })
    }

    pub fn get_type_for_name(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
}
