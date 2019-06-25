extern crate crossbeam_channel as channel;
extern crate graphql_parser;
extern crate serde_json;
use std::path::PathBuf;

mod graphql;
mod typescript;
mod work;
mod worker_pool;

fn main() {
    let schema_path = PathBuf::from("./schema.json");
    let schema = graphql::parse_schema(&schema_path).expect("Failed to parse schema");
    let worker_pool = worker_pool::WorkerPool::new(4, schema);
    let initial_work = work::Work::DirEntry(PathBuf::from(r"."));
    worker_pool.work(initial_work);
}
