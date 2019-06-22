use serde::Deserialize;
use serde_json::Result as SerdeResult;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct TypeMetaData {
    name: String,
    description: Option<String>,
}

pub enum Type {
    Object(TypeMetaData),
    Scalar(TypeMetaData),
}

impl Type {
    fn from(json: TypeJSON) -> Option<Self> {
        let metadata = TypeMetaData {
            name: json.name,
            description: json.description,
        };
        match json.kind.as_ref() {
            "INPUTOBJECT" | "OBJECT" => Some(Type::Object(metadata)),
            "SCALAR" => Some(Type::Scalar(metadata)),
            _ => None,
        }
    }
}

pub struct Schema {
    pub types: HashMap<String, Type>,
}

impl Schema {
    fn from(parsed: SchemaJSON) -> Self {
        let mut types = HashMap::with_capacity(parsed.types.len());
        for ty in parsed.types {
            let name = ty.name.clone();
            if let Some(processed_type) = Type::from(ty) {
                types.insert(name, processed_type);
            }
        }
        Schema { types }
    }
}

#[derive(Deserialize)]
struct TypeJSON {
    kind: String,
    name: String,
    description: Option<String>,
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

pub fn parse_schema(path: &PathBuf) -> SerdeResult<Schema> {
    let file = File::open(path).expect("Could not read schema file");
    let reader = BufReader::new(file);
    let parsed: RawSchemaJSON = serde_json::from_reader(reader)?;
    Ok(Schema::from(parsed.data.schema))
}

pub fn compile_file(path: &PathBuf) {}
