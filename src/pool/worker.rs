use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
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
    /// En caso de no poder leer del canal, espera un breve tiempo y vuelve a intentarlo.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Intentamos leer del canal con un pequeño timeout para no bloquear indefinidamente
            let job = {
                let receiver = receiver.lock().unwrap();
                receiver.recv_timeout(Duration::from_secs(2)).ok() // Espera 2 segundos entre intentos
            };

            match job {
                Some(job) => {
                    job();
                },
                None => {
                    // Si no hay trabajos después de 2 segundos, seguimos esperando
                },
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
