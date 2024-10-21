// Modulo que modela al Cliente en nuestro modelo Cliente-Servidor
// El cliente se comunica con un solo nodo del servidor
// El cliente envia consultas CQL al servidor y recibe la respuesta

use crate::messagestuff::message::Message;
use bufstream::BufStream;
use std::io::{self, BufRead, Result, Write};
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

    pub fn handle_input(&mut self) -> Result<()> {
        let mut _stream = TcpStream::connect(&self.server_ip)?;

        Ok(())
    }

    pub fn handle_connection(&self) -> Result<()> {
        let mut stream = TcpStream::connect(&self.server_ip)?;

        let stdin = io::stdin();
        let handle = stdin.lock();

        let stream_clone = stream.try_clone()?; 
        let reader = BufStream::new(stream_clone);

        // Thread que recibe la Tabla o el mensaje de "Recibido"
        // Solo un nodo del server se comunica con el cliente
        thread::spawn(move || {
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let message = Message::from_bytes(line.as_bytes());
                        println!("Received: {:?}", message);
                    }
                    // Si falla la lectura del stream se imprime el error
                    // Se deja de escuchar al nodo
                    Err(e) => {
                        eprintln!("Failed to read from stream: {}", e);
                        break;
                    }
                }
            }
        });

        // Loop que envia consultas CQL al server
        // Cliente solo se comunica con un Nodo del server
        for line in handle.lines() {
            let input = line?;
            let trimmed_input: &str = input.trim(); 

            if !trimmed_input.is_empty() {
                let mut mensaje = Message::create_message_as_client(trimmed_input.to_string()); // Create the message
                stream.write_all(&mensaje.to_bytes())?;
                stream.write_all(b"\n")?;
                stream.flush()?;
            }

            // Si el cliente escribe "exit" se cierra la conexión
            if trimmed_input == "exit" {
                break;
            }
        }

        Ok(())
    }
}
