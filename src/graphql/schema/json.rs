//! Deserialzing and handling introspection JSON
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
pub struct FieldSubtypeJSON {
    pub kind: String,
    pub name: Option<String>,
    #[serde(rename(deserialize = "ofType"))]
    pub of_type: Option<Box<FieldSubtypeJSON>>,
}

#[derive(Deserialize)]
pub struct FieldJSON {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub type_desc: FieldSubtypeJSON,
}

#[derive(Deserialize)]
pub struct EnumValuesJSON {
    pub name: String,
}

#[derive(Deserialize)]
pub struct TypeJSON {
    pub kind: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "inputFields"))]
    pub input_fields: Option<Vec<FieldJSON>>,
    pub fields: Option<Vec<FieldJSON>>,
    #[serde(rename(deserialize = "enumValues"))]
    pub enum_values: Option<Vec<EnumValuesJSON>>,
}

#[derive(Deserialize)]
pub struct SchemaJSON {
    pub types: Vec<TypeJSON>,
}

impl SchemaJSON {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, serde_json::Error> {
        let parsed: RawSchemaJSON = serde_json::from_reader(reader)?;
        Ok(parsed.data.schema)
    }
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
