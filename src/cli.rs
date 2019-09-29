use crate::graphql::BottomTypeConfig;
use crate::worker_pool::Error;
use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use std::convert::TryFrom;
use std::path::PathBuf;

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
        .get_matches()
}

/// User configured runtime configuration
#[derive(Debug)]
pub struct RuntimeConfig {
    root_dir: PathBuf,
    schema_path: PathBuf,
    use_custom_scalars: bool,
    custom_scalar_prefix: Option<String>,
    number_threads: u8,
}

impl RuntimeConfig {
    pub fn from_cli() -> Self {
        let matches = cli_parse();
        let root_dir = PathBuf::from(matches.value_of("root_dir").unwrap());
        let use_custom_scalars = matches.is_present("use_custom_scalars");
        let schema_path = match matches.value_of("schema_path") {
            Some(value) => PathBuf::from(value),
            None => {
                let mut path = root_dir.clone();
                path.push("schema.json");
                path
            }
        };
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
                .or_else(|| u8::try_from(num_cpus::get()).ok())
                .unwrap_or_else(|| 4),
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

    pub fn thread_count(&self) -> u8 {
        self.number_threads
    }
}

/// Prints the result of the programe to the screen.
pub fn print_work_result(result: Result<(), Vec<Error>>) {
    let code = match result {
        Ok(_) => return,
        Err(errors) => {
            for error in errors {
                eprintln!("{:?}", error);
            }
            1
        }
    };
    std::process::exit(code);
}
