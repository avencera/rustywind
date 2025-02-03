/// Heard is a struct that handles running the rustywind on a list of files
/// in parallel. It uses the num_cpus crate to determine the number of
/// physical cores on the machine. It then spawns a thread for each core
/// and runs the rustywind on the file paths.
use crate::options::Options;
use std::{path::PathBuf, sync::Arc};

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
        log::debug!("checking {} files", file_paths.len());

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
