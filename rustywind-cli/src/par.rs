use std::{path::PathBuf, sync::Arc};

use crossbeam::channel::{Receiver, Sender};

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct Actor {
    pub index: usize,
    pub total: usize,
    pub receiver: Receiver<Vec<PathBuf>>,
    pub options: Arc<Options>,
}

#[derive(Debug)]
pub struct Heard {
    pub senders: Vec<Sender<Vec<PathBuf>>>,
    pub actors: Vec<std::thread::JoinHandle<()>>,
}

impl Heard {
    pub fn new(options: Arc<Options>) -> Self {
        let physical_cores = num_cpus::get_physical();
        let mut actors = Vec::with_capacity(physical_cores);
        let mut senders = Vec::with_capacity(physical_cores);

        for index in 0..physical_cores {
            let (sender, receiver) = crossbeam::channel::bounded(1000);
            let actor = Actor::new(index, physical_cores, receiver, options.clone()).start();

            senders.push(sender);
            actors.push(actor);
        }

        Self { senders, actors }
    }

    pub fn run_on_file_paths(self, file_paths: Vec<PathBuf>) {
        let total_chunks = self.senders.len();
        let chunks_of = file_paths.len() / total_chunks;

        file_paths
            .chunks(chunks_of)
            .enumerate()
            .for_each(|(index, chunk)| {
                let senders = self.senders[index].clone();
                senders.send(chunk.to_vec()).unwrap();
            });

        self.complete();
    }

    pub fn complete(mut self) {
        // droping the senders will close the channels
        {
            let senders = std::mem::take(&mut self.senders);
            drop(senders);
        }

        // wait for all the threads to finish
        for actor in self.actors.into_iter() {
            actor.join().unwrap();
        }
    }
}

impl Actor {
    pub fn new(
        index: usize,
        total: usize,
        receiver: Receiver<Vec<PathBuf>>,
        options: Arc<Options>,
    ) -> Self {
        Self {
            index,
            total,
            receiver,
            options,
        }
    }

    pub fn start(self) -> std::thread::JoinHandle<()> {
        let options = self.options.clone();

        let receiver = self.receiver.clone();
        std::thread::spawn(move || {
            let file_paths = receiver.recv().unwrap();
            for file_path in file_paths {
                crate::run_on_file_paths(file_path, &options);
            }
        })
    }
}
