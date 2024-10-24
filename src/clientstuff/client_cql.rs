use crate::config;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::messagestuff::message_cql::querymessage::QueryMessage;
use crate::messagestuff::message::Sendable;
use std::io::{self, BufRead, BufReader, Write, Result};
use std::net::TcpStream;
use bufstream::BufStream;

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

    pub fn handle_input(&mut self) -> Result<()> {
        let mut _stream = TcpStream::connect(&self.server_ip)?;

        Ok(())
    }

    pub fn handle_connection_2(&self) -> io::Result<()> {
        let mut stream = match TcpStream::connect(&self.server_ip) {
            Ok(s) => s,
            Err(e) => {
                println!("Error al conectarme al TcpStream: {:?}", e);
                return Err(e);
            }
        };
        let mut stream_clone = match stream.try_clone() {
            Ok(clone) => clone,
            Err(e) => {
                println!("Error al clonar el TcpStream: {:?}", e);
                return Err(e);
            }
        };
        thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut input = String::new();
                if stdin.lock().read_line(& mut input).is_ok() {
                    if !input.is_empty() {
                        println!("Hola");
                        let msg: QueryMessage = QueryMessage{query: input, consistency: config::ONE};
                        let _ = msg.send_through(&mut stream_clone);
                        /*let env = msg.into_envelope();
                        if let Err(e) = stream_clone.write_all(&env.to_bytes()) {
                            println!("Error al escribir en el stream: {:?}", e);
                        }
                        if let Err(e) = stream_clone.flush() {
                            println!("Error al vecial es stream: {:?}", e);
                            break;
                        }*/
                    }
                }
            }
        });
        
        loop {
            match Envelope::from_stream(&mut stream) {
                Ok(env) => {
                    println!("El env recibido es: {:?}", env);
                }
                Err(e) => {
                    println!("Error al leer el env: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }
/*
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
                        let message = CqlMessage::from_bytes(line.as_bytes());
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
                let mensaje = CqlMessage::new(trimmed_input.to_string()); // Create the message
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
*/
}
