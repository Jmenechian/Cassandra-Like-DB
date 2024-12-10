use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
/// Representa los extemos de un channel por donde se puede enviar y recibir una tupla de un string y un u16.
pub struct ChannelType {
    pub sender: Sender<(String, u16)>, 
    pub receiver: Receiver<(String, u16)>, 
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelType {
    pub fn new() -> Self {
        let channel = mpsc::channel();
        Self {
            sender: channel.0,
            receiver: channel.1
        }
    }
}