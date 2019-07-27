use crate::worker_pool::Error;
use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub root_dir: PathBuf,
    pub schema_path: PathBuf,
    pub number_threads: u8,
}

fn cli_parse<'a>() -> ArgMatches<'a> {
    App::new("QL Compiler")
        .version(crate_version!())
        .author(crate_authors!())
        .about("\nQL Compiler (qlc) compiles type definitions from graphql and introspection JSON.")
        .arg(
            Arg::with_name("rootdir")
                .value_name("DIR")
                .default_value(".")
                .help("Directory to compile"),
        )
        .arg(
            Arg::with_name("schemapath")
                .value_name("FILE")
                .short("s")
                .long("schema-file")
                .takes_value(true)
                .help("Path of schema introspection JSON file"),
        )
        .arg(
            Arg::with_name("nthreads")
                .long("num-threads")
                .value_name("NUMBER")
                .takes_value(true)
                .validator(|val| val.parse::<u8>().map(|_| ()).map_err(|err| err.to_string()))
                .help("Sets the number of threads"),
        )
        .get_matches()
}

impl Config {
    pub fn from_cli() -> Self {
        let matches = cli_parse();
        let root_dir = PathBuf::from(matches.value_of("rootdir").unwrap());
        let schema_path = match matches.value_of("schemapath") {
            Some(value) => PathBuf::from(value),
            None => {
                let mut path = root_dir.clone();
                path.push("schema.json");
                path
            }
        };
        Config {
            root_dir,
            schema_path,
            number_threads: matches
                .value_of("nthreads")
                .and_then(|st| st.parse().ok())
                .or_else(|| u8::try_from(num_cpus::get()).ok())
                .unwrap_or_else(|| 4),
        }
    }
}

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
