use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    fs::{OpenOptions, File},
};

extern crate chrono;
use chrono::prelude::*;
// use std::thread;

#[derive(Debug)]
pub enum StatusNode {
    Connected,
    Desconnected,
}

#[derive(Debug)]
pub struct Node {
    pub status: StatusNode,
    pub ip: String,
    pub file_path: String,
    pub nodes_path: String,
    pub neighbors: Vec<Node>, //RF
    pub ring: Vec<Node>, //Gossip
}

impl Node {

    pub fn new(puerto: &str) -> Self {
        Self {
            status: StatusNode::Desconnected,
            ip: format!("127.0.0.1:{}",puerto),
            file_path: "".to_string(),
            nodes_path: "nodes.txt".to_string(),
            neighbors: Vec::new(),
            ring: Vec::new(),
        }
    }

    pub fn add_neighbor(&mut self) {
        // self.neighbors.push();
    }

    pub fn add_ring(&mut self) {
        let file = File::open(&self.nodes_path).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            if line != self.ip {
                let mut socket = TcpStream::connect(&line).unwrap();
                let algo = Local::now();
                let mensaje = format!("Hola soy {} vos sos {} y es la hora: {}", self.ip, line, algo);
                let _ = socket.write(mensaje.as_bytes());
                socket.flush().unwrap();
                // println!("\n\n\n\n\n\n{:?}\n\n\n\n\n\n\n", socket);
            }
        }
        // self.ring.push();
    }

    pub fn print_node(&self) {
        let mut file = OpenOptions::new().append(true).open(&self.nodes_path).unwrap();

        writeln!(file, "{}", self.ip).unwrap();

    }
    
    pub fn handle_connection(& mut self, stream: TcpStream) {
        
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
        while let Some(Ok(line)) = lines.next() {
            println!("Recibido: {:?}", line);
        }
    }

    pub fn connect_node(& mut self) {
        // para gossip y para conectar con cliente
        self.print_node();

        let listener = TcpListener::bind(&self.ip).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.add_ring();
            self.handle_connection(stream);
        }
    }
}

