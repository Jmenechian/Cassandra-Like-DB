use crate::pool::job::Job;
use crate::pool::worker::Worker;
use std::sync::{mpsc, Arc, Mutex};

/// Estructura que representa un threadpool con la cantidad de workers indicado.
/// Envia por un extremo del channel (sender), los jobs que deben realizar los threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Crea un ThreadPool.
    /// size indica la cantidad de hilos que se crean. Este parametro debe ser mayor a 0.
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Se envía por el channel el trabajo a realizar.
    /// Los workers estaran escuchando estos jobs del otro extremo del channel.
    /// Esta lectura estara lockeada para un correcto funcionamiento (no ejecutar una tarea más de una vez).
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
