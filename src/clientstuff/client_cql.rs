use crate::config;
use crate::messagestuff::message::Sendable;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::messagestuff::message_cql::querymessage::QueryMessage;
use crate::messagestuff::message_cql::resultmessage::ResultBody;
use crate::my_errors::MyError;
use std::io::{self, BufRead, Result};
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

    pub fn connect(&mut self) -> Result<TcpStream> {
        if let Ok(stream) = TcpStream::connect(&self.server_ip) {
            Ok(stream)
        } else {
            Err(MyError::TcpStreamError("Error connecting  c client to server".to_string()).into())
        }
    }

    pub fn handle_connection(&self) -> io::Result<()> {
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
                if stdin.lock().read_line(&mut input).is_ok() {
                    let _ = Client::send_message(&mut stream_clone, input);
                }
            }
        });

        loop {
            match Envelope::from_stream(&mut stream) {
                Ok(env) => {
                    //println!("El env recibido es: {:?}", env);
                    match env.body {
                        CqlMessage::Result(r) => match r.body {
                            ResultBody::Rows(r) => {
                                for row in r.rows_content {
                                    println!("{}", row);
                                }
                            }
                            ResultBody::Void(_) => {
                                println!("Error: Error en la consulta ingresada");
                            }
                        },
                        CqlMessage::Error(_) => {
                            println!("Error: Paso algo inesperado");
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("Error al leer el env: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn send_message(stream: &mut TcpStream, message: String) -> Result<()> {
        if !message.is_empty() {
            let msg: QueryMessage = QueryMessage {
                query: message,
                consistency: config::QUORUM,
            };
            let _ = msg.send_through(stream);
        }
        Ok(())
    }

    pub fn read_response(stream: &mut TcpStream) -> Result<Vec<String>> {
        loop {
            match Envelope::from_stream(stream) {
                Ok(env) => {
                    println!("El env recibido es: {:?}", env);
                    match env.body {
                        CqlMessage::Result(result) => match result.body {
                            ResultBody::Void(_) => {
                                return Ok(vec!["".to_string()]);
                            }
                            ResultBody::Rows(rows) => {
                                return Ok(rows.rows_content);
                            }
                        },
                        _ => {
                            continue;
                        }
                    }
                }
                Err(e) => {
                    return Err(MyError::Error(e.to_string()).into());
                }
            }
        }
    }
}
