extern crate crossbeam_channel as channel;
use std::path::PathBuf;
mod graphql;
mod work;
mod worker_pool;

fn main() {
    let worker_pool = worker_pool::WorkerPool::new(4);
    let initial_work = work::Work::DirEntry(PathBuf::from(r"."));
    worker_pool.work(initial_work);
}
