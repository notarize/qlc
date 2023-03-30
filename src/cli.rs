use crate::graphql::BottomTypeConfig;
use clap::Parser;
use colored::{control, Colorize};
use graphql_parser::Pos;
use serde::Deserialize;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Error as IOError};
use std::path::{Path, PathBuf};
use strsim::generic_damerau_levenshtein;

#[derive(Debug)]
enum MessageLevel {
    CompileWarning,
    CompileError,
    ProgramError,
}

#[derive(Debug, Clone)]
pub struct LocationInformation {
    line_number: String,
    line: String,
    help_text: Option<String>,
    column_number: Option<usize>,
}

impl LocationInformation {
    pub fn new_from_line_and_column(line_number: usize, line: &str, column_number: usize) -> Self {
        Self {
            line_number: line_number.to_string(),
            line: line.to_string(),
            column_number: Some(column_number),
            help_text: None,
        }
    }

    pub fn new_from_contents_and_position(contents: &str, position: &Pos) -> Self {
        Self {
            line_number: position.line.to_string(),
            line: contents
                .lines()
                .nth(position.line - 1)
                .unwrap_or("<<QLC unknown line>>")
                .to_string(),
            column_number: Some(position.column),
            help_text: None,
        }
    }

    pub fn with_help_text(&mut self, help_text: &str) -> &mut Self {
        self.help_text = Some(help_text.to_string());
        self
    }

    fn line_number_digits(&self) -> usize {
        self.line_number.len()
    }

    fn fmt_colon_encoding(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.column_number {
            None => write!(f, ":{}", self.line_number),
            Some(column_number) => write!(f, ":{}:{}", self.line_number, column_number),
        }
    }

    fn fmt_line_encoding(
        &self,
        f: &mut fmt::Formatter<'_>,
        colorize: impl Fn(&str) -> colored::ColoredString,
    ) -> fmt::Result {
        let bar = "|".blue();
        let indent_spaces = " ".repeat(self.line_number_digits() + 1);
        write!(
            f,
            "\n{}{}\n{} {} {}\n",
            indent_spaces,
            bar,
            self.line_number.blue(),
            bar,
            self.line
        )?;
        match self.column_number {
            None => write!(f, "{} {}", indent_spaces, bar),
            Some(column_number) => {
                let column_spaces = " ".repeat(column_number - 1);
                let arrow = colorize("^");
                write!(f, "{}{} {}{}", indent_spaces, bar, column_spaces, arrow)
            }
        }?;
        match self.help_text {
            None => Ok(()),
            Some(ref text) => write!(
                f,
                "\n{}{} {} {}",
                indent_spaces,
                "=".blue(),
                "help:".green(),
                text,
            ),
        }
    }
}

#[derive(Debug)]
struct SourceInformation {
    file_path: PathBuf,
    location: Option<LocationInformation>,
}

impl SourceInformation {
    fn fmt_file(
        &self,
        f: &mut fmt::Formatter<'_>,
        colorize: impl Fn(&str) -> colored::ColoredString,
    ) -> fmt::Result {
        let loc_ref = self.location.as_ref();
        let indent_spaces = " ".repeat(loc_ref.map(|loc| loc.line_number_digits()).unwrap_or(2));
        let file_name = self.file_path.display();
        write!(f, "\n{}{} {}", indent_spaces, "-->".blue(), file_name)?;
        match loc_ref {
            None => Ok(()),
            Some(loc) => {
                loc.fmt_colon_encoding(f)?;
                loc.fmt_line_encoding(f, colorize)
            }
        }
    }
}

#[derive(Debug)]
pub struct PrintableMessage {
    level: MessageLevel,
    source_information: Option<SourceInformation>,
    message: String,
}

impl fmt::Display for PrintableMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let colorize = match self.level {
            MessageLevel::CompileWarning => |str: &str| str.yellow(),
            _ => |str: &str| str.red(),
        };
        let header = match self.level {
            MessageLevel::CompileWarning => "warning".yellow(),
            MessageLevel::CompileError => "error".red(),
            MessageLevel::ProgramError => "program error".red(),
        };
        write!(f, "{}{} {}", header.bold(), ":".bold(), self.message.bold())?;
        match self.source_information {
            Some(ref info) => info.fmt_file(f, colorize),
            None => Ok(()),
        }
    }
}

impl PrintableMessage {
    pub fn new_compile_error_from_read_io_error(error: &IOError, path: &Path) -> Self {
        PrintableMessage::new_compile_error_from_io_error("read", error, path)
    }

    pub fn new_compile_error_from_write_io_error(error: &IOError, path: &Path) -> Self {
        PrintableMessage::new_compile_error_from_io_error("write", error, path)
    }

    pub fn new_simple_compile_error(message: &str) -> Self {
        PrintableMessage {
            level: MessageLevel::CompileError,
            message: String::from(message),
            source_information: None,
        }
    }

    pub fn new_simple_compile_warning(message: &str) -> Self {
        PrintableMessage {
            level: MessageLevel::CompileWarning,
            message: String::from(message),
            source_information: None,
        }
    }

    pub fn new_compile_error(
        message: &str,
        source_file_path: &Path,
        contents: &str,
        position: &Pos,
        help_text: Option<&str>,
    ) -> Self {
        let mut location = LocationInformation::new_from_contents_and_position(contents, position);
        if let Some(help_text) = help_text {
            location.with_help_text(help_text);
        }
        let source_information = SourceInformation {
            file_path: source_file_path.to_owned(),
            location: Some(location),
        };
        PrintableMessage {
            level: MessageLevel::CompileError,
            message: String::from(message),
            source_information: Some(source_information),
        }
    }

    pub fn new_program_error(
        message: &str,
        source_file_path: &Path,
        contents: &str,
        position: &Pos,
        help_text: Option<&str>,
    ) -> Self {
        let mut location = LocationInformation::new_from_contents_and_position(contents, position);
        if let Some(help_text) = help_text {
            location.with_help_text(help_text);
        }
        let source_information = SourceInformation {
            file_path: source_file_path.to_owned(),
            location: Some(location),
        };
        PrintableMessage {
            level: MessageLevel::ProgramError,
            message: String::from(message),
            source_information: Some(source_information),
        }
    }

    pub fn new_simple_program_error(message: &str) -> Self {
        PrintableMessage {
            level: MessageLevel::ProgramError,
            message: String::from(message),
            source_information: None,
        }
    }

    pub fn new_compile_warning(
        message: &str,
        source_file_path: &Path,
        contents: &str,
        position: &Pos,
        help_text: Option<&str>,
    ) -> Self {
        let mut location = LocationInformation::new_from_contents_and_position(contents, position);
        if let Some(help_text) = help_text {
            location.with_help_text(help_text);
        }
        let source_information = SourceInformation {
            file_path: source_file_path.to_owned(),
            location: Some(location),
        };
        PrintableMessage {
            level: MessageLevel::CompileWarning,
            message: String::from(message),
            source_information: Some(source_information),
        }
    }

    pub fn with_source_information(
        &mut self,
        source_file_path: &Path,
        location: Option<LocationInformation>,
    ) -> &mut Self {
        self.source_information = Some(SourceInformation {
            location,
            file_path: source_file_path.to_owned(),
        });
        self
    }

    fn new_compile_error_from_io_error(
        operation_name: &str,
        error: &std::io::Error,
        path: &Path,
    ) -> Self {
        PrintableMessage {
            level: MessageLevel::CompileError,
            message: format!("could not {operation_name} `{}`: {error}", path.display()),
            source_information: None,
        }
    }
}

pub trait ExitInformation {
    fn messages(&self) -> &[PrintableMessage];
}

impl ExitInformation for Vec<PrintableMessage> {
    fn messages(&self) -> &[PrintableMessage] {
        self
    }
}

#[derive(Parser, Debug)]
#[command(name = "QL Compiler", version)]
#[command(
    about = "QL Compiler (qlc) compiles type definitions from graphql and introspection JSON."
)]
struct CliArgs {
    /// Directory to recursively compile
    root_dir: Option<PathBuf>,
    /// Path of JSON configuration file
    #[arg(short, long, value_name = "FILE_PATH")]
    config_file: Option<PathBuf>,
    /// Path of schema introspection JSON file (defaults to <ROOT_DIR>/schema.json)
    #[arg(short, long, value_name = "FILE_PATH")]
    schema_path: Option<PathBuf>,
    /// Use custom schema defined scalar names for types instead of any type
    #[arg(long)]
    use_custom_scalars: bool,
    /// Prefix the name of custom scalars to keep them unique, requires --use-custom-scalars
    #[arg(long, value_name = "PREFIX", requires = "use_custom_scalars")]
    custom_scalar_prefix: Option<String>,
    /// Sets an import prefix for the root directory, ie `@/` to get allow `#import "@/fragment.graphql";`
    #[arg(long, value_name = "PREFIX")]
    root_dir_import_prefix: Option<String>,
    /// Sets the file name used for global enums and objects, defaults to `graphql-globals`
    #[arg(long, value_name = "MODULE_NAME")]
    global_types_module_name: Option<String>,
    /// Sets the module name that will be used for importing typed GraphQL document nodes, defaults to `@notarize/qlc-cli/typed-documentnode`
    #[arg(long, value_name = "MODULE_NAME")]
    typed_graphql_documentnode_module_name: Option<String>,
    /// Disable marking types as readonly
    #[arg(long)]
    disable_readonly_types: bool,
    /// Enables warnings for deprecated field usage
    #[arg(long)]
    show_deprecation_warnings: bool,
    /// Sets the number of threads (defaults to number of CPU cores)
    #[arg(long, value_name = "NUMBER")]
    num_threads: Option<usize>,
    /// Disables color output
    #[arg(long)]
    no_color: bool,
}

/// User configured configuration from configuration file, if it exists
#[derive(Debug, Default, Deserialize)]
struct ConfigFileMatches {
    #[serde(rename(deserialize = "schemaFile"))]
    schema_path: Option<PathBuf>,
    #[serde(rename(deserialize = "useCustomScalars"))]
    use_custom_scalars: Option<bool>,
    #[serde(rename(deserialize = "disableReadonlyTypes"))]
    disable_readonly_types: Option<bool>,
    #[serde(rename(deserialize = "customScalarPrefix"))]
    custom_scalar_prefix: Option<String>,
    #[serde(rename(deserialize = "numThreads"))]
    num_threads: Option<usize>,
    #[serde(rename(deserialize = "showDeprecationWarnings"))]
    show_deprecation_warnings: Option<bool>,
    #[serde(rename(deserialize = "rootDirImportPrefix"))]
    root_dir_import_prefix: Option<String>,
    #[serde(rename(deserialize = "globalTypesModuleName"))]
    global_types_module_name: Option<String>,
    #[serde(rename(deserialize = "typedGraphqlDocumentnodeModuleName"))]
    typed_graphql_documentnode_module_name: Option<String>,
}

impl ConfigFileMatches {
    fn from_file_parse(cli_file_arg: Option<&Path>) -> Result<Self, PrintableMessage> {
        let config_file_was_cli_provided = cli_file_arg.is_some();
        let config_file_path = cli_file_arg.unwrap_or_else(|| Path::new(".qlcrc.json"));
        match File::open(config_file_path) {
            Ok(file) => serde_json::from_reader(BufReader::new(file))
                .map(|mut config: Self| {
                    config.schema_path = config.schema_path.and_then(|schema_path| {
                        if schema_path.is_absolute() {
                            Some(schema_path)
                        } else {
                            config_file_path
                                .parent()
                                .map(|parent_config_dir| parent_config_dir.join(schema_path))
                        }
                    });
                    config
                })
                .map_err(|serde_error| {
                    PrintableMessage::new_simple_program_error(&format!(
                        "error in config file `{}`: {serde_error}",
                        config_file_path.display(),
                    ))
                }),
            Err(io_error) => {
                if config_file_was_cli_provided || io_error.kind() != std::io::ErrorKind::NotFound {
                    Err(PrintableMessage::new_compile_error_from_read_io_error(
                        &io_error,
                        config_file_path,
                    ))
                } else {
                    Ok(ConfigFileMatches::default())
                }
            }
        }
    }
}

/// User configured runtime configuration
#[derive(Debug)]
pub struct RuntimeConfig {
    root_dir: PathBuf,
    schema_path: PathBuf,
    show_deprecation_warnings: bool,
    use_custom_scalars: bool,
    disable_readonly_types: bool,
    custom_scalar_prefix: Option<String>,
    number_threads: usize,
    root_dir_import_prefix: Option<String>,
    global_types_module_name: String,
    typed_graphql_documentnode_module_name: String,
}

impl RuntimeConfig {
    pub fn from_cli() -> Self {
        let cli_args = CliArgs::parse();

        if cli_args.no_color {
            control::set_override(false);
        }

        let config_file_args = ConfigFileMatches::from_file_parse(cli_args.config_file.as_deref())
            .unwrap_or_else(|config_error_message| {
                print_exit_info(vec![config_error_message]);
            });

        let root_dir = cli_args.root_dir.unwrap_or_else(|| PathBuf::from("."));
        let schema_path = cli_args
            .schema_path
            .or(config_file_args.schema_path)
            .unwrap_or_else(|| root_dir.join("schema.json"));

        RuntimeConfig {
            root_dir,
            schema_path,
            show_deprecation_warnings: cli_args.show_deprecation_warnings
                || config_file_args.show_deprecation_warnings.unwrap_or(false),
            use_custom_scalars: cli_args.use_custom_scalars
                || config_file_args.use_custom_scalars.unwrap_or(false),
            disable_readonly_types: cli_args.disable_readonly_types
                || config_file_args.disable_readonly_types.unwrap_or(false),
            custom_scalar_prefix: cli_args.custom_scalar_prefix.or_else(|| {
                config_file_args
                    .use_custom_scalars
                    .and(config_file_args.custom_scalar_prefix)
            }),
            number_threads: cli_args
                .num_threads
                .or(config_file_args.num_threads)
                .unwrap_or_else(|| std::cmp::min(num_cpus::get(), 8)),
            root_dir_import_prefix: cli_args
                .root_dir_import_prefix
                .or(config_file_args.root_dir_import_prefix),
            global_types_module_name: cli_args
                .global_types_module_name
                .or(config_file_args.global_types_module_name)
                .unwrap_or_else(|| String::from("graphql-globals")),
            typed_graphql_documentnode_module_name: cli_args
                .typed_graphql_documentnode_module_name
                .or(config_file_args.typed_graphql_documentnode_module_name)
                .unwrap_or_else(|| String::from("@notarize/qlc-cli/typed-documentnode")),
        }
    }

    pub fn root_dir_path(&self) -> PathBuf {
        self.root_dir.clone()
    }

    pub fn schema_file_path(&self) -> PathBuf {
        self.schema_path.clone()
    }

    pub fn bottom_type_config(&self) -> BottomTypeConfig {
        match (self.use_custom_scalars, &self.custom_scalar_prefix) {
            (false, _) => BottomTypeConfig::DefaultBottomType,
            (true, None) => BottomTypeConfig::RealName,
            (true, Some(s)) => BottomTypeConfig::RealNameWithPrefix(s.clone()),
        }
    }

    pub fn show_deprecation_warnings(&self) -> bool {
        self.show_deprecation_warnings
    }

    pub fn thread_count(&self) -> usize {
        self.number_threads
    }

    pub fn root_dir_import_prefix(&self) -> Option<String> {
        self.root_dir_import_prefix.clone()
    }

    pub fn global_types_module_name(&self) -> String {
        self.global_types_module_name.clone()
    }

    pub fn typed_graphql_documentnode_module_name(&self) -> String {
        self.typed_graphql_documentnode_module_name.clone()
    }

    pub fn disable_readonly_types(&self) -> bool {
        self.disable_readonly_types
    }
}

pub fn similar_help_suggestions(
    comparison_name: &str,
    possible_names: impl Iterator<Item = String>,
) -> Option<String> {
    let comparison_name = comparison_name.as_ref();
    let similar_names: Vec<_> = possible_names
        .filter(|possible_field_name| {
            generic_damerau_levenshtein(comparison_name, possible_field_name.as_ref()) < 3
        })
        .collect();
    if similar_names.is_empty() {
        None
    } else {
        Some(format!(
            " Did you mean one of the following: `{}`?",
            similar_names.join("`, `")
        ))
    }
}

/// Prints the result of the program to the screen with process exiting.
pub fn print_exit_info(exit_info: impl ExitInformation) -> ! {
    let mut warning_count = 0;
    let mut error_count = 0;
    for msg in exit_info.messages() {
        match msg.level {
            MessageLevel::CompileWarning => {
                warning_count += 1;
            }
            MessageLevel::CompileError | MessageLevel::ProgramError => {
                error_count += 1;
            }
        }
        println!("{}\n", msg);
    }
    let has_errors = error_count > 0;
    if has_errors {
        let plural = if error_count > 1 { "s" } else { "" };
        println!(
            "{}",
            PrintableMessage::new_simple_compile_error(&format!(
                "failure due to {error_count} error{plural}",
            ))
        );
    }
    if warning_count > 0 {
        let plural = if error_count > 1 { "s" } else { "" };
        println!(
            "{}",
            PrintableMessage::new_simple_compile_warning(&format!(
                "{warning_count} warning{plural} emitted",
            ))
        );
    }

    std::process::exit(i32::from(has_errors));
}
