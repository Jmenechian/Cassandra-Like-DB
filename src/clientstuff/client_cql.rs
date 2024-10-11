use crate::messagestuff::message::Message;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use bufstream::BufStream;
use std::thread;
const REQUEST: u8 = 0x03;
const RESPONSE: u8 = 0x83;


#[derive(Debug)]
pub struct Client {
    pub server_ip: String,
    pub ip: String,
}

impl Client {
    pub fn initialize(port: String) -> Self {
        Self {
            server_ip: format!("127.0.0.1:800{}", port),
            ip: format!("127.0.0.1:45450"), //randint?
        }
    }

    pub fn handle_input(&mut self) -> io::Result<()> {
        let mut stream = TcpStream::connect(&self.server_ip)?;
        


        Ok(())
    }

    pub fn handle_connection(&self) -> io::Result<()> {
        let mut stream = TcpStream::connect(&self.server_ip)?;
    
        let stdin = io::stdin();
        let handle = stdin.lock();
        
        // Spawn a thread to listen for incoming messages from the stream
        let mut stream_clone = stream.try_clone()?; // Clone the stream for the listener thread
        let reader = BufReader::new(stream_clone); // Wrap the cloned stream in a BufReader
    
        thread::spawn(move || {
            let mut buffer = String::new();
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let message = Message::from_bytes(line.as_bytes());
                        println!("Received: {:?}", message);
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stream: {}", e);
                        break;
                    }
                }
            }
        });
    
        // Main loop for sending messages
        for line in handle.lines() {
            let input = line?;
            let trimmed_input = input.trim(); // Trim whitespace from input
            
            if !trimmed_input.is_empty() {
                let mensaje = Message::new(trimmed_input.to_string()); // Create the message
                stream.write_all(&mensaje.to_bytes())?; 
                stream.write_all(b"\n")?; 
                stream.flush()?; 
            }
    
            if trimmed_input == "exit" {
                break;
            }
        }
    
        Ok(())
    }

}

