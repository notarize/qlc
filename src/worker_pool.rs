use super::graphql::schema::Schema;
use super::work::{compile_global_file, Work};
use std::collections::HashSet;
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

struct Worker {
    threads: usize,
    schema: Arc<Schema>,
    is_waiting: bool,
    is_quitting: bool,
    global_types: Arc<Mutex<HashSet<String>>>,
    num_waiting: Arc<AtomicUsize>,
    num_quitting: Arc<AtomicUsize>,
    tx: channel::Sender<Message>,
    rx: channel::Receiver<Message>,
}

impl Worker {
    fn run(mut self) {
        while let Some(work) = self.pop_work() {
            let (new_work, used_globals) = work.run(&self.schema);
            if let Some(works) = new_work {
                for work in works {
                    self.tx.send(Message::Work(work)).unwrap();
                }
            }
            if let Some(globals) = used_globals {
                let mut self_globals = self.global_types.lock().unwrap();
                for global in globals {
                    self_globals.insert(global);
                }
            }
        }
    }

    fn pop_work(&mut self) -> Option<super::work::Work> {
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
    num_workers: usize,
    schema: Arc<Schema>,
}

impl WorkerPool {
    pub fn new(num_workers: usize, schema: Schema) -> Self {
        WorkerPool {
            num_workers,
            schema: Arc::new(schema),
        }
    }

    pub fn work(&self, root_dir: PathBuf) {
        let threads = self.num_workers;
        let global_types = Arc::new(Mutex::new(HashSet::new()));
        let (tx, rx) = channel::unbounded();
        let num_waiting = Arc::new(AtomicUsize::new(0));
        let num_quitting = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        for _ in 0..threads {
            let worker = Worker {
                threads,
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
        let initial_work = Work::DirEntry(root_dir.clone());
        let root = Message::Work(initial_work);
        tx.send(root).unwrap();
        drop(tx);
        drop(rx);
        for handle in handles {
            handle.join().unwrap();
        }

        let global_types = global_types.lock().unwrap();
        compile_global_file(&root_dir, &self.schema, &global_types);
    }
}
