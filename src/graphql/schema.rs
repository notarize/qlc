use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;

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
pub struct FieldMetaData {
    pub nullable: bool,
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
    fn from(name: &str) -> Self {
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
pub struct InterfaceType {
    pub possible_types: Vec<String>,
}

#[derive(Debug)]
pub enum FieldType {
    List(FieldMetaData, Box<FieldType>),
    Object(FieldMetaData, String),
    Enum(FieldMetaData, EnumType),
    Scalar(FieldMetaData, ScalarType),
}

impl FieldType {
    fn from(json: FieldSubtypeJSON) -> Result<Self, Error> {
        let mut metadata = FieldMetaData { nullable: true };
        let mut iter = json;
        loop {
            match iter.kind.as_ref() {
                "NON_NULL" => {
                    metadata.nullable = false;
                    iter = *iter.of_type.ok_or_else(|| Error::MissingTypeOfForNonNull)?;
                }
                "LIST" => {
                    iter = *iter.of_type.ok_or_else(|| Error::MissingTypeOfForList)?;
                    let field_type = FieldType::from(iter)?;
                    return Ok(FieldType::List(metadata, Box::new(field_type)));
                }
                "OBJECT" | "INTERFACE" => {
                    let name = iter.name.ok_or_else(|| Error::MissingNameForField)?;
                    return Ok(FieldType::Object(metadata, name));
                }
                "SCALAR" => {
                    let name = iter.name.ok_or_else(|| Error::MissingNameForField)?;
                    return Ok(FieldType::Scalar(metadata, ScalarType::from(&name)));
                }
                _ => return Ok(FieldType::Scalar(metadata, ScalarType::Boolean)), // TODO
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
    fn from(json: FieldJSON) -> Self {
        let type_desc = json.type_desc;
        Field {
            name: json.name,
            description: json.description,
            type_description: FieldType::from(type_desc).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct ObjectType {
    pub fields: HashMap<String, Field>,
}

#[derive(Debug)]
pub struct EnumType {
    pub possible_values: Vec<String>,
}

#[derive(Debug)]
pub struct TypeMetaData {
    pub description: Option<String>,
}

#[derive(Debug)]
pub enum Type {
    Object(TypeMetaData, ObjectType),
    Enum(TypeMetaData, EnumType),
    Scalar(TypeMetaData, String),
}

impl Type {
    fn from(json: TypeJSON) -> Result<Self, Error> {
        let metadata = TypeMetaData {
            description: json.description,
        };
        match json.kind.as_ref() {
            "OBJECT" => {
                let fields_json = json.fields.unwrap_or_else(Vec::new);
                let mut fields = HashMap::with_capacity(fields_json.len());
                for field_json in fields_json {
                    fields.insert(field_json.name.clone(), Field::from(field_json));
                }
                let object_type = ObjectType { fields };
                Ok(Type::Object(metadata, object_type))
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
                Ok(Type::Enum(metadata, enum_type))
            }
            "SCALAR" => Ok(Type::Scalar(metadata, json.name)),
            // TODO
            "INPUT_OBJECT" | "UNION" | "INTERFACE" => Ok(Type::Scalar(metadata, json.name)),
            _ => Err(Error::UnknownTypeKind(json.name, json.kind)),
        }
    }
}

#[derive(Deserialize)]
struct FieldSubtypeJSON {
    kind: String,
    name: Option<String>,
    #[serde(rename(deserialize = "ofType"))]
    of_type: Option<Box<FieldSubtypeJSON>>,
}

#[derive(Deserialize)]
struct FieldJSON {
    name: String,
    description: Option<String>,
    #[serde(rename(deserialize = "type"))]
    type_desc: FieldSubtypeJSON,
}

#[derive(Deserialize)]
struct EnumValuesJSON {
    name: String,
}

#[derive(Deserialize)]
struct TypeJSON {
    kind: String,
    name: String,
    description: Option<String>,
    fields: Option<Vec<FieldJSON>>,
    #[serde(rename(deserialize = "enumValues"))]
    enum_values: Option<Vec<EnumValuesJSON>>,
}

#[derive(Deserialize)]
struct SchemaJSON {
    types: Vec<TypeJSON>,
}

#[derive(Deserialize)]
struct DataJSON {
    #[serde(rename(deserialize = "__schema"))]
    schema: SchemaJSON,
}

#[derive(Deserialize)]
struct RawSchemaJSON {
    data: DataJSON,
}

pub struct Schema {
    types: HashMap<String, Type>,
}

impl Schema {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let parsed: RawSchemaJSON =
            serde_json::from_reader(reader).map_err(Error::JSONParseError)?;
        let parsed_schema = parsed.data.schema;
        let mut types = HashMap::with_capacity(parsed_schema.types.len());
        for ty in parsed_schema.types {
            let name = ty.name.clone();
            let processed_type = Type::from(ty)?;
            types.insert(name, processed_type);
        }
        Ok(Schema { types })
    }

    pub fn get_type_for_name(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
}
