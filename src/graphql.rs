use super::cli::{LocationInformation, PrintableMessage, RuntimeConfig};
use crate::typescript;
use graphql_parser::query::{Definition, Document, FragmentDefinition, OperationDefinition};
use schema::Schema;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

pub mod ir;
pub mod schema;
pub mod variable;

const IMPORT_START: &str = "#import \"";

pub type ParsedTextType = String;

#[derive(Debug)]
pub struct CompileReport {
    pub messages: Vec<PrintableMessage>,
    pub global_types_used: HashSet<String>,
}

#[derive(Debug)]
pub enum BottomTypeConfig {
    DefaultBottomType,
    RealName,
    RealNameWithPrefix(String),
}

#[derive(Debug)]
pub struct CompileConfig {
    root_dir: PathBuf,
    show_deprecation_warnings: bool,
    pub use_readonly_types: bool,
    pub bottom_type_config: BottomTypeConfig,
    pub root_dir_import_prefix: Option<String>,
    pub global_types_module_name: String,
    pub generated_module_name: String,
}

impl From<&RuntimeConfig> for CompileConfig {
    fn from(from: &RuntimeConfig) -> Self {
        CompileConfig {
            root_dir: from.root_dir_path(),
            use_readonly_types: !from.disable_readonly_types(),
            bottom_type_config: from.bottom_type_config(),
            show_deprecation_warnings: from.show_deprecation_warnings(),
            root_dir_import_prefix: from.root_dir_import_prefix(),
            global_types_module_name: from.global_types_module_name(),
            generated_module_name: from.generated_module_name(),
        }
    }
}

fn read_graphql_file(path: &Path) -> Result<String, PrintableMessage> {
    File::open(path)
        .and_then(|file| {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;
            Ok(contents)
        })
        .map_err(|io_error| PrintableMessage::new_compile_error_from_read_io_error(&io_error, path))
}

fn parse_graphql_file<'a>(
    contents: &'a str,
    file_path: &'_ Path,
) -> Result<Document<'a, ParsedTextType>, PrintableMessage> {
    let parsed = graphql_parser::parse_query(contents).map_err(|parse_error| {
        // TODO parse error has no line/column information
        let mut error = PrintableMessage::new_simple_compile_error(&format!("{parse_error}"));
        error.with_source_information(file_path, None);
        error
    })?;
    let num_defintions = parsed.definitions.len();
    if num_defintions == 0 {
        let message =
            PrintableMessage::new_simple_compile_error("missing defintion in the document");
        return Err(message);
    } else if num_defintions > 1 {
        let mut message =
            PrintableMessage::new_simple_compile_error("multi definition files unsupported");
        let position = match &parsed.definitions[1] {
            Definition::Operation(op) => match op {
                OperationDefinition::SelectionSet(set) => &set.span.0,
                OperationDefinition::Query(q) => &q.position,
                OperationDefinition::Mutation(m) => &m.position,
                OperationDefinition::Subscription(s) => &s.position,
            },
            Definition::Fragment(frag) => &frag.position,
        };
        let mut location = LocationInformation::new_from_contents_and_position(contents, position);
        location.with_help_text("QLC does not support documents with more than one fragment, query or mutation per file. Move this definition to a seperate file and import it instead.");
        message.with_source_information(file_path, Some(location));
        return Err(message);
    }
    Ok(parsed)
}

fn makedir_p(path: &Path) -> Result<(), PrintableMessage> {
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(ref io_error) if io_error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(io_error) => Err(PrintableMessage::new_compile_error_from_write_io_error(
            &io_error, path,
        )),
    }
}

fn make_generated_dir(
    config: &CompileConfig,
    mut path: PathBuf,
) -> Result<PathBuf, PrintableMessage> {
    path.push(&config.generated_module_name);
    makedir_p(&path)?;
    Ok(path)
}

fn get_file_path_of_fragment(
    config: &CompileConfig,
    import_comment: &str,
    current_dir: &Path,
) -> PathBuf {
    let last_quote = import_comment
        .rfind('"')
        .unwrap_or(import_comment.len() - 1);
    let import_path = &import_comment[IMPORT_START.len()..last_quote];

    config
        .root_dir_import_prefix
        .as_deref()
        .and_then(|root_dir_prefix| import_path.strip_prefix(root_dir_prefix))
        .map(|import_path_without_prefix| config.root_dir.join(import_path_without_prefix))
        .unwrap_or_else(|| {
            if import_path.starts_with('.') {
                current_dir.join(import_path)
            } else {
                config.root_dir.join(import_path)
            }
        })
}

fn add_imported_fragments(
    config: &CompileConfig,
    current_file: &Path,
    current_file_contents: &str,
    import_contents: &mut HashMap<PathBuf, (String, LocationInformation)>,
    messages: &mut Vec<PrintableMessage>,
) {
    let mut current_dir = current_file.to_owned();
    current_dir.pop();
    for (line_index, line) in current_file_contents.lines().enumerate() {
        if line.is_empty() || line.trim().is_empty() {
            continue;
        }
        let doesnt_start_with_import = !line.starts_with(IMPORT_START);
        if doesnt_start_with_import && line.starts_with('#') {
            // We keep going for comments
            continue;
        } else if doesnt_start_with_import {
            // Stop reading lines as soon as we find a non-comment, non-empty line
            break;
        }
        let location =
            LocationInformation::new_from_line_and_column(line_index + 1, line, IMPORT_START.len());
        let file_path = get_file_path_of_fragment(config, line, &current_dir);
        let other_file_contents = match read_graphql_file(&file_path) {
            Ok(c) => c,
            Err(mut sub_message) => {
                sub_message.with_source_information(current_file, Some(location));
                messages.push(sub_message);
                continue;
            }
        };
        add_imported_fragments(
            config,
            &file_path,
            &other_file_contents,
            import_contents,
            messages,
        );
        import_contents.insert(file_path, (other_file_contents, location));
    }
}

fn parse_foreign_fragments<'a>(
    path: &Path,
    imported_contents: &'a HashMap<PathBuf, (String, LocationInformation)>,
    messages: &mut Vec<PrintableMessage>,
) -> HashMap<String, FragmentDefinition<'a, ParsedTextType>> {
    let mut parsed_imported_fragments = HashMap::new();
    for (other_path, (contents, location)) in imported_contents.iter() {
        if let Ok(mut parsed) = parse_graphql_file(contents, other_path) {
            // We already know there is exactly one definition since read_graphql_file
            // throws away documents that contain more or less than one defintion.
            for def in parsed.definitions.drain(0..1) {
                match def {
                    Definition::Fragment(f_def) => {
                        let fragment_name = f_def.name.clone();
                        parsed_imported_fragments.insert(fragment_name, f_def);
                    }
                    _ => {
                        let mut message = PrintableMessage::new_simple_compile_error(&format!(
                            "cannot import non-fragment GraphQL document `{}`",
                            path.display()
                        ));
                        let mut location = location.clone();
                        location.with_help_text("This document is not a fragment, and importing it is probably a mistake.");
                        message.with_source_information(path, Some(location));
                        messages.push(message);
                    }
                }
            }
        }
    }
    parsed_imported_fragments
}

pub fn compile_file(
    path: &Path,
    config: &CompileConfig,
    schema: &Schema,
) -> Result<CompileReport, Vec<PrintableMessage>> {
    let contents = read_graphql_file(path).map_err(|e| vec![e])?;
    let parsed = parse_graphql_file(&contents, path).map_err(|e| vec![e])?;

    let mut messages = Vec::new();
    let mut imported_contents = HashMap::new();
    add_imported_fragments(
        config,
        path,
        &contents,
        &mut imported_contents,
        &mut messages,
    );
    let parsed_imported_fragments =
        parse_foreign_fragments(path, &imported_contents, &mut messages);

    let (op_ir, warnings) = match ir::Operation::compile(
        &parsed.definitions[0],
        schema,
        parsed_imported_fragments,
        config.show_deprecation_warnings,
    ) {
        Ok(ir) => ir,
        Err((ir_errors, warnings)) => {
            messages.extend(
                ir_errors
                    .into_iter()
                    .map(|ir_error| PrintableMessage::from((contents.as_ref(), path, ir_error)))
                    .chain(warnings.into_iter().map(|ir_warning| {
                        PrintableMessage::from((contents.as_ref(), path, ir_warning))
                    })),
            );
            return Err(messages);
        }
    };

    messages.extend(
        warnings
            .into_iter()
            .map(|ir_warning| PrintableMessage::from((contents.as_ref(), path, ir_warning))),
    );

    let the_compile = match typescript::compile_ir(&op_ir, config, schema) {
        Ok(c) => c,
        Err(inner_message) => {
            messages.push(inner_message.into());
            return Err(messages);
        }
    };

    let mut parent_dir = path.to_path_buf();
    parent_dir.pop();
    let mut generated_dir_path = match make_generated_dir(config, parent_dir) {
        Ok(path) => path,
        Err(error) => {
            messages.push(error);
            return Err(messages);
        }
    };
    generated_dir_path.push(the_compile.filename);
    std::fs::write(&generated_dir_path, the_compile.contents).map_err(|io_error| {
        vec![PrintableMessage::new_compile_error_from_write_io_error(
            &io_error,
            &generated_dir_path,
        )]
    })?;

    generated_dir_path.pop();
    Ok(CompileReport {
        messages,
        global_types_used: the_compile.global_types_used,
    })
}

pub fn compile_global_types_file(
    path: &Path,
    config: &CompileConfig,
    schema: &Schema,
    global_names: &HashSet<String>,
) -> Result<(), PrintableMessage> {
    if global_names.is_empty() {
        return Ok(());
    }
    let mut generated_dir_path = make_generated_dir(config, path.to_path_buf())?;
    let the_compile = typescript::compile_globals(config, schema, global_names)?;
    generated_dir_path.push(the_compile.filename);
    std::fs::write(&generated_dir_path, the_compile.contents).map_err(|io_error| {
        PrintableMessage::new_compile_error_from_write_io_error(&io_error, &generated_dir_path)
    })?;
    Ok(())
}
