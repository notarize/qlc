use super::cli::RuntimeConfig;
use super::graphql::schema::Schema;
use super::graphql::{compile_file, compile_global_types_file, CompileConfig};
use crossbeam_channel as channel;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    GraphQL(super::graphql::Error),
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

    fn run(&self, config: &CompileConfig, schema: &Arc<Schema>) -> WorkResult {
        match self {
            Work::DirEntry(path) => self
                .run_dir_entry(path)
                .map(SuccessWorkResult::MoreWork)
                .map_err(Error::IO),
            Work::GraphQL(path) => compile_file(path, config, schema)
                .map(SuccessWorkResult::MoreGlobalTypes)
                .map_err(Error::GraphQL),
        }
    }
}

struct Worker {
    schema: Arc<Schema>,
    is_waiting: bool,
    is_quitting: bool,
    global_types: Arc<Mutex<HashSet<String>>>,
    errors: Arc<Mutex<Vec<Error>>>,
    num_waiting: Arc<AtomicUsize>,
    num_quitting: Arc<AtomicUsize>,
    tx: channel::Sender<Message>,
    rx: channel::Receiver<Message>,
    config: Arc<RuntimeConfig>,
}

impl Worker {
    fn run(mut self) {
        let compile_config = CompileConfig::from(&*self.config);
        while let Some(work) = self.pop_work() {
            match work.run(&compile_config, &self.schema) {
                Ok(SuccessWorkResult::MoreGlobalTypes(new_globals)) => {
                    let mut self_globals = self.global_types.lock().unwrap();
                    for potential_new_global in new_globals.into_iter() {
                        self_globals.insert(potential_new_global);
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
                        let threads = self.config.thread_count();
                        // If the number of waiting workers dropped, then abort our attempt to quit.
                        // Sometimes work will come back.
                        if nwait < threads {
                            break;
                        }
                        // If all workers are in this quit loop, then we can stop.
                        if nquit == threads {
                            return None;
                        }
                    }
                }
                Err(_) => {
                    let threads = self.config.thread_count();
                    self.set_waiting(true);
                    self.set_quitting(false);
                    if self.num_waiting() == threads {
                        for _ in 0..threads {
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

    fn num_waiting(&self) -> usize {
        self.num_waiting.load(Ordering::SeqCst)
    }

    fn num_quitting(&self) -> usize {
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
    config: Arc<RuntimeConfig>,
    schema: Arc<Schema>,
}

impl WorkerPool {
    pub fn new(config: RuntimeConfig, schema: Schema) -> Self {
        WorkerPool {
            config: Arc::new(config),
            schema: Arc::new(schema),
        }
    }

    pub fn work(&self) -> Result<(), Vec<Error>> {
        let global_types = Arc::new(Mutex::new(HashSet::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = channel::unbounded();
        let num_waiting = Arc::new(AtomicUsize::new(0));
        let num_quitting = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        let initial_work = Work::DirEntry(self.config.root_dir_path());
        let root = Message::Work(initial_work);
        tx.send(root).unwrap();
        for _ in 0..self.config.thread_count() {
            let worker = Worker {
                errors: errors.clone(),
                schema: self.schema.clone(),
                num_quitting: num_quitting.clone(),
                num_waiting: num_waiting.clone(),
                global_types: global_types.clone(),
                is_quitting: false,
                is_waiting: false,
                tx: tx.clone(),
                rx: rx.clone(),
                config: self.config.clone(),
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
        if let Err(global_type_error) = compile_global_types_file(
            &self.config.root_dir_path(),
            &CompileConfig::from(&*self.config),
            &self.schema,
            &global_types,
        ) {
            errors.push(Error::GraphQL(global_type_error));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
