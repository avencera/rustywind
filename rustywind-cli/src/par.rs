use std::{path::PathBuf, sync::Arc};

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct Actor {
    pub index: usize,
    pub total: usize,
    pub options: Arc<Options>,
}

#[derive(Debug)]
pub struct Heard {
    cpus: usize,
    options: Arc<Options>,
}

impl Heard {
    pub fn new(options: Arc<Options>) -> Self {
        let cpus = num_cpus::get_physical();
        Self { cpus, options }
    }

    pub fn run_on_file_paths(self, file_paths: Vec<PathBuf>) {
        let total_chunks = self.cpus;
        let chunks_of = file_paths.len() / total_chunks;
        let options = &self.options;

        std::thread::scope(|s| {
            file_paths.chunks(chunks_of).for_each(|chunk| {
                s.spawn(|| {
                    run_on_file_paths(chunk, options);
                });
            });
        })
    }
}

fn run_on_file_paths(file_paths: &[PathBuf], options: &Options) {
    for file_path in file_paths {
        crate::run_on_file_path(file_path, options);
    }
}
