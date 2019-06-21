extern crate crossbeam_channel as channel;
mod worker_pool;

fn main() {
    let worker_pool = worker_pool::WorkerPool::new(4);
    worker_pool.work();
}
