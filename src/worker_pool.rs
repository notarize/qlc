use super::graphql;
use super::graphql::schema::Schema;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Message {
    Work(Work),
    Quit,
}

#[derive(Debug)]
pub enum Error {
    CleanupError,
    GraphQL(graphql::Error),
    IO(std::io::Error),
}

#[derive(Debug)]
enum SuccessWorkResult {
    MoreWork(Vec<Work>),
    MoreGlobalTypes(HashSet<String>),
}

type WorkResult = Result<SuccessWorkResult, Error>;

#[derive(Debug)]
enum Work {
    GraphQL(PathBuf),
    DirEntry(PathBuf),
}

impl Work {
    fn run_dir_entry(&self, path: &PathBuf) -> Result<Vec<Work>, std::io::Error> {
        let readdir = fs::read_dir(path)?;
        let mut more_work = vec![];
        for raw_entry in readdir {
            let path = raw_entry?.path();
            if path.is_dir() {
                more_work.push(Work::DirEntry(path));
            } else if path.is_file() && path.extension().map_or(false, |x| x == "graphql") {
                more_work.push(Work::GraphQL(path));
            }
        }
        Ok(more_work)
    }

    fn run(&self, schema: &Arc<Schema>, root_dir: &Arc<PathBuf>) -> WorkResult {
        match self {
            Work::DirEntry(path) => self
                .run_dir_entry(path)
                .map(SuccessWorkResult::MoreWork)
                .map_err(Error::IO),
            Work::GraphQL(path) => super::graphql::compile_file(path, schema, root_dir)
                .map(SuccessWorkResult::MoreGlobalTypes)
                .map_err(Error::GraphQL),
        }
    }
}

struct Worker {
    threads: u8,
    schema: Arc<Schema>,
    root_dir: Arc<PathBuf>,
    is_waiting: bool,
    is_quitting: bool,
    global_types: Arc<Mutex<HashSet<String>>>,
    errors: Arc<Mutex<Vec<Error>>>,
    num_waiting: Arc<AtomicU8>,
    num_quitting: Arc<AtomicU8>,
    tx: channel::Sender<Message>,
    rx: channel::Receiver<Message>,
}

impl Worker {
    fn run(mut self) {
        while let Some(work) = self.pop_work() {
            match work.run(&self.schema, &self.root_dir) {
                Ok(SuccessWorkResult::MoreGlobalTypes(globals)) => {
                    let mut self_globals = self.global_types.lock().unwrap();
                    for global in globals {
                        self_globals.insert(global);
                    }
                }
                Ok(SuccessWorkResult::MoreWork(works)) => {
                    for work in works {
                        self.tx.send(Message::Work(work)).unwrap();
                    }
                }
                Err(e) => {
                    let mut all_errors = self.errors.lock().unwrap();
                    all_errors.push(e);
                }
            }
        }
    }

    fn pop_work(&mut self) -> Option<Work> {
        loop {
            match self.rx.try_recv() {
                Ok(Message::Work(work)) => {
                    self.set_waiting(false);
                    self.set_quitting(false);
                    return Some(work);
                }
                Ok(Message::Quit) => {
                    self.set_waiting(true);
                    self.set_quitting(true);
                    loop {
                        let nwait = self.num_waiting();
                        let nquit = self.num_quitting();
                        // If the number of waiting workers dropped, then abort our attempt to quit.
                        // Sometimes work will come back.
                        if nwait < self.threads {
                            break;
                        }
                        // If all workers are in this quit loop, then we can stop.
                        if nquit == self.threads {
                            return None;
                        }
                    }
                }
                Err(_) => {
                    self.set_waiting(true);
                    self.set_quitting(false);
                    if self.num_waiting() == self.threads {
                        for _ in 0..self.threads {
                            self.tx.send(Message::Quit).unwrap();
                        }
                    } else {
                        // This is a bit weird, I know, but we want producers to catch up
                        // without burning the CPU too hard.
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        }
    }

    fn num_waiting(&self) -> u8 {
        self.num_waiting.load(Ordering::SeqCst)
    }

    fn num_quitting(&self) -> u8 {
        self.num_quitting.load(Ordering::SeqCst)
    }

    fn set_waiting(&mut self, desired: bool) {
        if desired && !self.is_waiting {
            self.is_waiting = true;
            self.num_waiting.fetch_add(1, Ordering::SeqCst);
        } else if !desired && self.is_waiting {
            self.is_waiting = false;
            self.num_waiting.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn set_quitting(&mut self, desired: bool) {
        if desired && !self.is_quitting {
            self.is_quitting = true;
            self.num_quitting.fetch_add(1, Ordering::SeqCst);
        } else if !desired && self.is_quitting {
            self.is_quitting = false;
            self.num_quitting.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

pub struct WorkerPool {
    num_workers: u8,
    schema: Arc<Schema>,
}

impl WorkerPool {
    pub fn new(num_workers: u8, schema: Schema) -> Self {
        WorkerPool {
            num_workers,
            schema: Arc::new(schema),
        }
    }

    pub fn work(&self, root_dir: &PathBuf) -> Result<(), Vec<Error>> {
        let threads = self.num_workers;
        let global_types = Arc::new(Mutex::new(HashSet::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = channel::unbounded();
        let num_waiting = Arc::new(AtomicU8::new(0));
        let num_quitting = Arc::new(AtomicU8::new(0));
        let shared_root_dir = Arc::new(root_dir.clone());
        let mut handles = vec![];
        let initial_work = Work::DirEntry(root_dir.clone());
        let root = Message::Work(initial_work);
        tx.send(root).unwrap();
        for _ in 0..threads {
            let worker = Worker {
                threads,
                root_dir: shared_root_dir.clone(),
                errors: errors.clone(),
                schema: self.schema.clone(),
                num_quitting: num_quitting.clone(),
                num_waiting: num_waiting.clone(),
                global_types: global_types.clone(),
                is_quitting: false,
                is_waiting: false,
                tx: tx.clone(),
                rx: rx.clone(),
            };
            let handle = thread::spawn(|| worker.run());
            handles.push(handle);
        }
        drop(tx);
        drop(rx);
        for handle in handles {
            handle.join().unwrap();
        }

        let global_types = global_types.lock().unwrap();
        let mut errors = Arc::try_unwrap(errors)
            .map_err(|_| vec![Error::CleanupError])?
            .into_inner()
            .map_err(|_| vec![Error::CleanupError])?;
        if let Err(global_type_error) =
            graphql::compile_global_types_file(root_dir, &self.schema, &global_types)
        {
            errors.push(Error::GraphQL(global_type_error));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
