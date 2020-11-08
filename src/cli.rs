use crate::graphql::BottomTypeConfig;
use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use colored::{control, Colorize};
use graphql_parser::Pos;
use std::fmt;
use std::path::{Path, PathBuf};
use strsim::generic_damerau_levenshtein;

#[derive(Debug)]
enum MessageLevel {
    CompileWarning,
    CompileError,
    ProgramError,
}

#[derive(Debug)]
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
    pub fn new_compile_error_from_read_io_error(error: &std::io::Error, path: &Path) -> Self {
        PrintableMessage::new_compile_error_from_io_error("read", error, path)
    }

    pub fn new_compile_error_from_write_io_error(error: &std::io::Error, path: &Path) -> Self {
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
            message: format!(
                "could not {} `{}`: {}",
                operation_name,
                path.display(),
                error
            ),
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

fn cli_parse<'a>() -> ArgMatches<'a> {
    App::new("QL Compiler")
        .version(crate_version!())
        .author(crate_authors!())
        .about("\nQL Compiler (qlc) compiles type definitions from graphql and introspection JSON.")
        .arg(
            Arg::with_name("root_dir")
                .value_name("DIR")
                .default_value(".")
                .help("Directory to recursively compile"),
        )
        .arg(
            Arg::with_name("schema_path")
                .takes_value(true)
                .value_name("FILE")
                .short("s")
                .long("schema-file")
                .help("Path of schema introspection JSON file (defaults to DIR/schema.json)"),
        )
        .arg(
            Arg::with_name("use_custom_scalars")
                .long("use-custom-scalars")
                .help("Use custom schema defined scalar names for types instead of any type"),
        )
        .arg(
            Arg::with_name("custom_scalar_prefix")
                .takes_value(true)
                .value_name("PREFIX")
                .requires("use_custom_scalars")
                .long("custom-scalar-prefix")
                .help("Prefix the name of custom scalars to keep them unique"),
        )
        .arg(
            Arg::with_name("nthreads")
                .long("num-threads")
                .value_name("NUMBER")
                .takes_value(true)
                .validator(|val| val.parse::<u8>().map(|_| ()).map_err(|err| err.to_string()))
                .help("Sets the number of threads (defaults to number of CPU cores)"),
        )
        .arg(
            Arg::with_name("no_color")
                .long("no-color")
                .help("Use this flag to disable colors"),
        )
        .get_matches()
}

/// User configured runtime configuration
#[derive(Debug)]
pub struct RuntimeConfig {
    root_dir: PathBuf,
    schema_path: PathBuf,
    use_custom_scalars: bool,
    custom_scalar_prefix: Option<String>,
    number_threads: usize,
}

impl RuntimeConfig {
    pub fn from_cli() -> Self {
        let matches = cli_parse();
        if matches.is_present("no_color") {
            control::set_override(false);
        }
        let root_dir = PathBuf::from(matches.value_of("root_dir").unwrap());
        let use_custom_scalars = matches.is_present("use_custom_scalars");
        let schema_path = matches
            .value_of("schema_path")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut path = root_dir.clone();
                path.push("schema.json");
                path
            });
        RuntimeConfig {
            root_dir,
            schema_path,
            use_custom_scalars,
            custom_scalar_prefix: matches
                .value_of("custom_scalar_prefix")
                .map(|s| s.to_string()),
            number_threads: matches
                .value_of("nthreads")
                .and_then(|st| st.parse().ok())
                .unwrap_or_else(|| std::cmp::min(num_cpus::get(), 8)),
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
            (false, _) => BottomTypeConfig::UseBottomType,
            (true, None) => BottomTypeConfig::UseRealName,
            (true, Some(s)) => BottomTypeConfig::UseRealNameWithPrefix(s.clone()),
        }
    }

    pub fn thread_count(&self) -> usize {
        self.number_threads
    }
}

pub fn similar_help_suggestions(
    comparison_name: &str,
    possible_names: impl Iterator<Item = String>,
) -> Option<String> {
    let comparison_name = comparison_name.as_ref();
    let similar_names: Vec<_> = possible_names
        .filter(|possible_field_name| {
            generic_damerau_levenshtein(comparison_name, possible_field_name.as_ref()) < 5
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
pub fn print_exit_info(exit_info: impl ExitInformation) {
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
                "failure due to {} error{}",
                error_count, plural
            ))
        );
    }
    if warning_count > 0 {
        let plural = if error_count > 1 { "s" } else { "" };
        println!(
            "{}",
            PrintableMessage::new_simple_compile_warning(&format!(
                "{} warning{} emitted",
                warning_count, plural
            ))
        );
    }
    if has_errors {
        std::process::exit(1);
    }
}
