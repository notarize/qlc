use super::graphql::schema::Schema;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub enum Work {
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

    pub fn run(&self, schema: &Arc<Schema>) -> Vec<Work> {
        match self {
            Work::DirEntry(path) => self.run_dir_entry(path).unwrap_or_else(|_| vec![]),
            Work::GraphQL(path) => {
                match super::graphql::compile_file(path, schema) {
                    Ok(val) => {
                        dbg!(val);
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
                vec![]
            }
        }
    }
}
