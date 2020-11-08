#![deny(rust_2018_idioms)]

mod cli;
mod graphql;
mod typescript;
mod worker_pool;

fn main() {
    let config = cli::RuntimeConfig::from_cli();
    match graphql::schema::parse_schema(&config.schema_file_path()) {
        Ok(schema) => {
            let worker_pool = worker_pool::WorkerPool::new(config, schema);
            let work_aggregate = worker_pool.work();
            cli::print_exit_info(work_aggregate);
        }
        Err(schema_errors) => {
            cli::print_exit_info(schema_errors);
        }
    }
}
