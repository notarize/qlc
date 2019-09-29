extern crate clap;
extern crate crossbeam_channel as channel;
extern crate graphql_parser;
extern crate num_cpus;
extern crate serde_json;

mod cli;
mod graphql;
mod typescript;
mod worker_pool;

fn main() {
    let config = cli::RuntimeConfig::from_cli();
    let schema = graphql::parse_schema(&config.schema_file_path()).expect("Failed to parse schema");
    let worker_pool = worker_pool::WorkerPool::new(config, schema);
    let result = worker_pool.work();
    cli::print_work_result(result);
}
