use graphql_parser::query::{Definition, FragmentDefinition};
use schema::Schema;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

pub mod schema;

#[derive(Debug)]
pub enum Error {
    FileError(std::io::Error),
    SchemaJSONParseError(schema::Error),
    GraphqlFileParseError(graphql_parser::query::ParseError),
    OnlyImportFragments,
    CompileError(super::typescript::Error),
    OnlyOneOperationPerDocumentSupported,
}

fn read_graphql_file(path: &PathBuf) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn parse_schema(path: &PathBuf) -> Result<Schema, Error> {
    let file = File::open(path).map_err(Error::FileError)?;
    let reader = BufReader::new(file);
    Schema::from_reader(reader).map_err(Error::SchemaJSONParseError)
}

fn makedir_p(path: &PathBuf) -> Result<(), Error> {
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(Error::FileError(e)),
    }
}

fn make_generated_dir(mut path: PathBuf) -> Result<PathBuf, Error> {
    path.push("__generated__");
    makedir_p(&path)?;
    Ok(path)
}

fn get_file_path_of_fragment(
    import_comment: &str,
    current_dir: &PathBuf,
    root_dir: &PathBuf,
) -> PathBuf {
    let last_quote = import_comment
        .rfind('"')
        .unwrap_or(import_comment.len() - 1);
    let import_filename = &import_comment[9..last_quote];
    if import_filename.starts_with('.') {
        return current_dir.clone().join(import_filename);
    }
    root_dir.clone().join(import_filename)
}

fn add_imported_fragments(
    current_dir: &PathBuf,
    imports: &mut HashMap<String, FragmentDefinition>,
    contents: &str,
    root_dir: &PathBuf,
) -> Result<(), Error> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("#import \"") {
            continue;
        }
        let mut file_path = get_file_path_of_fragment(trimmed, current_dir, root_dir);
        let contents = read_graphql_file(&file_path).map_err(Error::FileError)?;
        file_path.pop();
        add_imported_fragments(&file_path, imports, &contents, root_dir)?;
        let mut parsed =
            graphql_parser::parse_query(&contents).map_err(Error::GraphqlFileParseError)?;
        if parsed.definitions.len() != 1 {
            return Err(Error::OnlyOneOperationPerDocumentSupported);
        }
        for def in parsed.definitions.drain(0..1) {
            match def {
                Definition::Fragment(f_def) => {
                    let fragment_name = f_def.name.clone();
                    imports.insert(fragment_name, f_def);
                }
                _ => return Err(Error::OnlyImportFragments),
            }
        }
    }
    Ok(())
}

pub fn compile_file(
    path: &PathBuf,
    schema: &Schema,
    root_dir: &PathBuf,
) -> Result<HashSet<String>, Error> {
    let contents = read_graphql_file(path).map_err(Error::FileError)?;
    let parsed = graphql_parser::parse_query(&contents).map_err(Error::GraphqlFileParseError)?;
    let mut parsed_imported_fragments = HashMap::new();
    let mut parent_dir = path.clone();
    parent_dir.pop();
    add_imported_fragments(
        &parent_dir,
        &mut parsed_imported_fragments,
        &contents,
        root_dir,
    )?;

    if parsed.definitions.len() != 1 {
        return Err(Error::OnlyOneOperationPerDocumentSupported);
    }

    let mut generated_dir_path = make_generated_dir(parent_dir)?;
    let the_compile =
        super::typescript::compile(&parsed.definitions[0], schema, parsed_imported_fragments)
            .map_err(Error::CompileError)?;
    generated_dir_path.push(the_compile.filename);
    std::fs::write(&generated_dir_path, the_compile.contents).map_err(Error::FileError)?;
    generated_dir_path.pop();
    Ok(the_compile.used_global_types)
}

pub fn compile_global_types_file(
    path: &PathBuf,
    schema: &Schema,
    global_names: &HashSet<String>,
) -> Result<(), Error> {
    if global_names.is_empty() {
        return Ok(());
    }
    let mut generated_dir_path = make_generated_dir(path.clone())?;
    let the_compile =
        super::typescript::compile_globals(schema, global_names).map_err(Error::CompileError)?;
    generated_dir_path.push(the_compile.filename);
    std::fs::write(&generated_dir_path, the_compile.contents).map_err(Error::FileError)?;
    Ok(())
}
