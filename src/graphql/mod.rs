use schema::Schema;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

mod document;
pub mod schema;
mod typescript;

#[derive(Debug)]
pub enum Error {
    FileReadError(std::io::Error),
    FileWriteError(typescript::Error),
    SchemaJSONParseError(schema::Error),
    GraphqlFileParseError(graphql_parser::query::ParseError),
    CompileError(document::Error),
}

fn read_graphql_file(path: &PathBuf) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn parse_schema(path: &PathBuf) -> Result<Schema, Error> {
    let file = File::open(path).map_err(Error::FileReadError)?;
    let reader = BufReader::new(file);
    Schema::from_reader(reader).map_err(Error::SchemaJSONParseError)
}

pub fn compile_file(path: &PathBuf, schema: &Schema) -> Result<(), Error> {
    let contents = read_graphql_file(path).map_err(Error::FileReadError)?;
    let parsed = graphql_parser::parse_query(&contents).map_err(Error::GraphqlFileParseError)?;
    let definition =
        dbg!(document::make_document_defs(parsed, schema).map_err(Error::CompileError)?);
    typescript::write(&definition, path).map_err(Error::FileWriteError)
}
