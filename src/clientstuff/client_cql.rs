use crate::messagestuff::message::Message;
use bufstream::BufStream;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::thread;

#[derive(Debug)]
pub struct Client {
    pub server_ip: String,
    pub ip: String,
}

impl Client {
    pub fn initialize(port: String) -> Self {
        Self {
            server_ip: format!("127.0.0.1:800{}", port),
            ip: "127.0.0.1:45450".to_string(), //randint?
        }
    }

    pub fn handle_input(&mut self) -> io::Result<()> {
        let mut _stream = TcpStream::connect(&self.server_ip)?;

        Ok(())
    }

    pub fn handle_connection(&self) -> io::Result<()> {
        let mut stream = TcpStream::connect(&self.server_ip)?;

        let stdin = io::stdin();
        let handle = stdin.lock();

        // Spawn a thread to listen for incoming messages from the stream
        let stream_clone = stream.try_clone()?; // Clone the stream for the listener thread
        let reader = BufStream::new(stream_clone);

        thread::spawn(move || {
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
                let mut mensaje = Message::create_message_as_client(trimmed_input.to_string()); // Create the message
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
