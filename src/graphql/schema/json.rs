//! Deserialzing and handling introspection JSON
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct FieldType {
    pub kind: String,
    pub name: Option<String>,
    #[serde(rename(deserialize = "ofType"))]
    pub of_type: Option<Box<FieldType>>,
}

#[derive(Deserialize, Debug)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub type_information: FieldType,
    #[serde(rename(deserialize = "isDeprecated"))]
    pub deprecated: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ComplexObjectDescription {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Type {
    pub kind: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "possibleTypes"))]
    pub possible_types: Option<Vec<ComplexObjectDescription>>,
    #[serde(rename(deserialize = "inputFields"))]
    pub input_fields: Option<Vec<Field>>,
    pub fields: Option<Vec<Field>>,
    #[serde(rename(deserialize = "enumValues"))]
    pub enum_values: Option<Vec<ComplexObjectDescription>>,
}

#[derive(Deserialize, Debug)]
pub struct Schema {
    pub types: Vec<Type>,
}

impl Schema {
    pub fn try_from_reader(reader: impl Read) -> Result<Self, serde_json::Error> {
        let parsed: RawSchema = serde_json::from_reader(reader)?;
        Ok(parsed.data.schema)
    }
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename(deserialize = "__schema"))]
    schema: Schema,
}

#[derive(Deserialize)]
struct RawSchema {
    data: Data,
}
