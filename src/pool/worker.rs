use std::{sync::{mpsc, Arc, Mutex}, thread};

use super::job::Job;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv();
    
                match message {
                    Ok(job) => {    
                        job();
                    }
                    Err(_) => {
                    }
                }
            });
    
            Worker {
                id,
                thread: Some(thread),
            }
        }
}