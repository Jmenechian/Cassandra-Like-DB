use super::{job::Job, worker::Worker};
use crate::errors::my_errors::MyError;

use std::sync::{mpsc, Arc, Mutex};

/// Estructura que representa un threadpool con la cantidad de workers indicado.
/// Envia por un extremo del channel (sender), los jobs que deben realizar los threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Cre un TreadPool.
    /// size indica la cantidad de hilos que se crean. Este parametro debe ser mayor a 0.
    pub fn build(size: usize) -> Result<ThreadPool, MyError> {
        if size <= 0 {
            return Err(MyError::ThreadError(
                "Cantidad de pool menor o igual a 0.".to_string(),
            ));
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let worker = Worker::build(id, Arc::clone(&receiver))?;
            workers.push(worker);
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
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
