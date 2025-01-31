use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use super::job::Job;
use crate::errors::my_errors::MyError;

/// Estructura que representa un worker (hilo en ejecución),
/// que escucha de un extremo del channel los jobs que debe realizar.
pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Se crea un worker.
    /// En caso de no poder leer del canal, se devulve un error.
    /// Esto indica que el channel ya no existe y se debe desconevtar el worker.
    pub fn build(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker, MyError> {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Ok(Worker {
            id,
            thread: Some(thread),
        })
    }
}
