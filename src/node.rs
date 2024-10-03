use std::{
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    fs::{OpenOptions, File},
};

extern crate chrono;
use chrono::prelude::*;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
// use std::thread;
use rand::prelude::*;

#[derive(Debug)]
pub enum StatusNode {
    Connected,
    Desconnected,
}

#[derive(Debug)]
pub struct Node {
    pub status: StatusNode,
    pub ip: String, //temp
    pub server_ip: String,
    pub client_ip: String,
    pub file_path: String,
    pub nodes_path: String,
    pub neighbors: Vec<Node>, //RF
    pub ring: Vec<String>, //Gossip //ip solo?
}

impl Node {

    pub fn new(puerto: &str) -> Self {
        Self {
            status: StatusNode::Desconnected,
            ip: format!("127.0.0.1:{}",puerto),
            server_ip: format!("127.0.0.{}:7000",puerto),
            client_ip: format!("127.0.0.{}:9042",puerto),
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

        let mut rng = rand::thread_rng();
        let mut nums: Vec<i32> = (0..8).collect();
        nums.shuffle(&mut rng);
        // println!("{:?}", nums);
        nums = nums[0..2].to_vec();
        // println!("{:?}", nums);

        for (index, line) in reader.lines().enumerate() {
            // println!("{:?}", line   );
            // if !nums.contains(&(index as i32)) {
            //     continue
            // }
            let line = line.unwrap();
            if line != self.server_ip && !self.ring.contains(&line) { 
                let mut socket = TcpStream::connect(&line).unwrap(); //conecto
                let msg = format!("Hola soy {} y me conecte a vos que sos: {}", self.server_ip, line);
                let _ = socket.write(msg.as_bytes());
                self.ring.push(line); //guardo tu ip
            }
        }
    }

    pub fn write_nodes(&self) {
        let file = OpenOptions::new().read(true).open(&self.nodes_path).unwrap();
        let reader = BufReader::new(file);
        let mut ip_exists = false;

        for line in reader.lines() {
            let line = line.unwrap();
            if line == self.server_ip { //si ip ya estan en el archivo chau
                ip_exists = true;
                break;
            }
        }

        if !ip_exists {
            let mut file = OpenOptions::new().append(true).open(&self.nodes_path).unwrap();
            writeln!(file, "{}", self.server_ip).unwrap();
        }
    }
    
    pub fn handle_connection(& mut self, stream: TcpStream) { //🤝
        let reader = BufReader::new(&stream);
        let mut lines = reader.lines();
        while let Some(Ok(line)) = lines.next() {

            // println!("Protocolo: {:?}", stream); // addr -> server escucha / peer -> cliente / fd -> file descriptor sirve para algo aca?

            println!("Recibido: {}", line); //mensaje del cliente
        }
    }

    pub fn connect_node2(& mut self) { //implementacion con thread, hay comunicacion server/server y client/server pero no podemos acceder a self (ver despues)
        // para gossip y para conectar con cliente
        self.write_nodes();

        let listener = TcpListener::bind(&self.ip).unwrap();

        let neighbor_ip = if self.ring.is_empty() {
            ""
        } else {
            &self.ring[0]
        };

        let msg = format!("Hola soy {}, vos sos {} y la fecha es {}\n", self.ip, neighbor_ip, Local::now()); 
        let stdin_thread = thread::spawn(move || {  //?
            //buscar otra manera / ver como hacer para manejar threads e instancias de self
            func(msg);
        });

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.add_ring(); //solo conecta
            self.handle_connection(stream); //lee lo del cliente
        }
    }

    pub fn send_message(&mut self, stream: &TcpStream) {
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        
        while let Some(Ok(line)) = lines.next() {
            if !line.is_empty() {
            let mut socket = TcpStream::connect(&self.ring[0]).unwrap(); //conecto
            let msg = format!("Hola soy {} y mi mensaje es: {}", self.server_ip, line);
            let _ = socket.write(msg.as_bytes());
            }
        }
    }

    pub fn connect_node(&mut self) { // Version limpia, no hay comunicacion server/server
        // Para gossip y para conectar con cliente
        self.write_nodes();
    
        let client_listener = TcpListener::bind(&self.client_ip).unwrap();
        client_listener.set_nonblocking(true).expect("Cannot set non-blocking");
        
        let server_listener = TcpListener::bind(&self.server_ip).unwrap();
        server_listener.set_nonblocking(true).expect("Cannot set non-blocking");
    
        loop {
            
            // Handling server listener
            match server_listener.accept() {
                Ok((stream, _)) => {
                    // println!("Server connection to server established");
                    // println!("{:?}", stream);
                    self.add_ring();
                    // self.send_message(&stream);
                    self.handle_connection(stream); // Lee lo del cliente
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.add_ring();
                    // println!("todavia no me conecte al server");
                    // No server connection ready, continue to the next iteration
                }
                Err(e) => {
                    eprintln!("Error accepting server connection: {:?}", e);
                }
            }
            // Handling client listener
            match client_listener.accept() {
                Ok((stream, _)) => {
                    println!("Client connection to client established");
                    self.handle_connection(stream); // Lee lo del cliente
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No client connection ready, continue to the next iteration
                }
                Err(e) => {
                    eprintln!("Error accepting client connection: {:?}", e);
                }
            }

            // Sleep for a bit to avoid busy waiting
            thread::sleep(Duration::from_millis(100));
        }
    }
    

}

pub fn func(msg:  String) {
    let stdin = io::stdin(); //input() para leer lo del server
            for line in stdin.lock().lines() {
                let input = line.unwrap();
                // let mi_ip = self.ip;
                let mut socket = TcpStream::connect("127.0.0.1:7879").unwrap();
                let new_msg = format!("{} AAAAAAAAAAAA{}", msg, Local::now());
                let _ = socket.write(new_msg.as_bytes());
                // let _ = socket.write(input.as_bytes());
                // socket.flush().unwrap();
            }
}