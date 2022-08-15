use super::cli::{ExitInformation, PrintableMessage, RuntimeConfig};
use super::graphql::schema::Schema;
use super::graphql::{compile_file, compile_global_types_file, CompileConfig};
use crossbeam_channel as channel;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Message {
    Work(Work),
    Quit,
}

#[derive(Debug)]
enum WorkResult {
    MoreWork(Vec<Work>),
    CompileResult {
        global_types_used: HashSet<String>,
        messages: Vec<PrintableMessage>,
    },
    DirIoError(std::io::Error, PathBuf),
}

#[derive(Debug)]
struct WorkAggregateResult {
    messages: Vec<PrintableMessage>,
    global_types: HashSet<String>,
}

impl WorkAggregateResult {
    fn new() -> Self {
        WorkAggregateResult {
            messages: Vec::new(),
            global_types: HashSet::new(),
        }
    }

    fn extend_globals(&mut self, new_globals: HashSet<String>) {
        self.global_types.extend(new_globals);
    }

    fn extend_messages(&mut self, messages: Vec<PrintableMessage>) {
        self.messages.extend(messages);
    }

    fn append_message(&mut self, message: PrintableMessage) {
        self.messages.push(message);
    }

    fn extend_from(&mut self, aggregate: Self) {
        let Self {
            messages,
            global_types,
        } = aggregate;
        self.extend_messages(messages);
        self.extend_globals(global_types);
    }
}

impl From<PrintableMessage> for WorkAggregateResult {
    fn from(message: PrintableMessage) -> Self {
        WorkAggregateResult {
            messages: vec![message],
            global_types: HashSet::new(),
        }
    }
}

impl ExitInformation for WorkAggregateResult {
    fn messages(&self) -> &[PrintableMessage] {
        &self.messages
    }
}

#[derive(Debug)]
enum Work {
    GraphQl(PathBuf),
    DirEntry(PathBuf),
}

impl Work {
    fn run_dir_entry(&self, path: &Path) -> Result<Vec<Work>, std::io::Error> {
        let readdir = fs::read_dir(path)?;
        let mut more_work = vec![];
        for raw_entry in readdir {
            let path = raw_entry?.path();
            if path.is_dir() {
                more_work.push(Work::DirEntry(path));
            } else if path.is_file() && path.extension().map_or(false, |x| x == "graphql") {
                more_work.push(Work::GraphQl(path));
            }
        }
        Ok(more_work)
    }

    fn run(&self, config: &CompileConfig, schema: &Schema) -> WorkResult {
        match self {
            Work::DirEntry(path) => self
                .run_dir_entry(path)
                .map(WorkResult::MoreWork)
                .unwrap_or_else(|io_error| WorkResult::DirIoError(io_error, path.clone())),
            Work::GraphQl(path) => compile_file(path, config, schema)
                .map(|compile_report| WorkResult::CompileResult {
                    global_types_used: compile_report.global_types_used,
                    messages: compile_report.messages,
                })
                .unwrap_or_else(|messages| WorkResult::CompileResult {
                    global_types_used: HashSet::new(),
                    messages,
                }),
        }
    }
}

struct Worker<'a> {
    pool: &'a WorkerPool,
    is_waiting: bool,
    is_quitting: bool,
    aggregate: WorkAggregateResult,
    num_waiting: &'a AtomicUsize,
    num_quitting: &'a AtomicUsize,
    tx: channel::Sender<Message>,
    rx: channel::Receiver<Message>,
}

impl<'a> Worker<'a> {
    fn run(mut self) -> WorkAggregateResult {
        while let Some(work) = self.pop_work() {
            match work.run(&self.pool.compile_config, &self.pool.schema) {
                WorkResult::CompileResult {
                    global_types_used,
                    messages,
                } => {
                    self.aggregate.extend_globals(global_types_used);
                    self.aggregate.extend_messages(messages);
                }
                WorkResult::MoreWork(additional_work) => {
                    for work in additional_work {
                        self.tx.send(Message::Work(work)).unwrap();
                    }
                }
                WorkResult::DirIoError(io_error, path) => {
                    self.aggregate.append_message(
                        PrintableMessage::new_compile_error_from_read_io_error(&io_error, &path),
                    );
                }
            }
        }
        self.aggregate
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
                        let threads = self.pool.thread_count;
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
                    let threads = self.pool.thread_count;
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
    compile_config: CompileConfig,
    root_dir_path: PathBuf,
    schema: Schema,
    thread_count: usize,
}

impl WorkerPool {
    pub fn new(runtime_config: RuntimeConfig, schema: Schema) -> Self {
        WorkerPool {
            compile_config: CompileConfig::from(&runtime_config),
            root_dir_path: runtime_config.root_dir_path(),
            schema,
            thread_count: runtime_config.thread_count(),
        }
    }

    pub fn work(&self) -> impl ExitInformation {
        let num_waiting = AtomicUsize::new(0);
        let num_quitting = AtomicUsize::new(0);

        let mut aggregate = thread::scope(|s| {
            let (tx, rx) = channel::unbounded();
            let initial_work = Message::Work(Work::DirEntry(self.root_dir_path.clone()));
            tx.send(initial_work).unwrap();

            let handles = (0..self.thread_count)
                .map(|_| {
                    let worker = Worker {
                        pool: self,
                        num_quitting: &num_quitting,
                        num_waiting: &num_waiting,
                        aggregate: WorkAggregateResult::new(),
                        is_quitting: false,
                        is_waiting: false,
                        tx: tx.clone(),
                        rx: rx.clone(),
                    };
                    s.spawn(|| worker.run())
                })
                .collect::<Vec<_>>();

            let mut aggregate = WorkAggregateResult::new();
            for handle in handles {
                aggregate.extend_from(handle.join().unwrap());
            }
            aggregate
        });

        if let Err(global_type_error) = compile_global_types_file(
            &self.root_dir_path,
            &self.compile_config,
            &self.schema,
            &aggregate.global_types,
        ) {
            aggregate.append_message(global_type_error);
        }

        aggregate
    }
}
