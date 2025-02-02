use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use super::job::Job;

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
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            if let Ok(m) = receiver.lock() {
                if let Ok(job) = m.recv() {
                    job();
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
