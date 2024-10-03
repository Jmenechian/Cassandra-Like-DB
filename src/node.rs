use std::{
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    fs::{OpenOptions, File},
};

extern crate chrono;
use chrono::prelude::*;
use std::thread;
use std::time::Duration;
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
    pub ip: String,
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
        println!("{:?}", nums);
        nums = nums[0..2].to_vec();
        println!("{:?}", nums);

        for (index, line) in reader.lines().enumerate() {
            if !nums.contains(&(index as i32)) {
                continue
            }
            let line = line.unwrap();
            if line != self.ip && !self.ring.contains(&line) { 
                TcpStream::connect(&line).unwrap(); //conecto
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
            if line == self.ip { //si ip ya estan en el archivo chau
                ip_exists = true;
                break;
            }
        }

        if !ip_exists {
            let mut file = OpenOptions::new().append(true).open(&self.nodes_path).unwrap();
            writeln!(file, "{}", self.ip).unwrap();
        }
    }
    
    pub fn handle_connection(& mut self, stream: TcpStream) { //🤝
        
        let reader = BufReader::new(&stream);
        let mut lines = reader.lines();
        while let Some(Ok(line)) = lines.next() {

            println!("Protocolo: {:?}", stream); // addr -> server escucha / peer -> cliente / fd -> file descriptor sirve para algo aca?

            if line == "connect" { //solo funciona una vez? 
                for ip in &self.ring {
                    let mut socket = TcpStream::connect(ip).unwrap();
                    let mensaje = format!("Hola soy {} y me quiero conectar a vos que sos: {}", self.ip, ip);
                    let _ = socket.write(mensaje.as_bytes());
                }
            }
            println!("Recibido: {}", line); //mensaje del cliente
        }
    }

    // pub fn connect_node2(& mut self) { //implementacion con thread, hay comunicacion server/server y client/server pero no podemos acceder a self (ver despues)
    //     // para gossip y para conectar con cliente
    //     self.write_nodes();

    //     let listener = TcpListener::bind(&self.ip).unwrap();
    //     let neighbor_ip = if self.ring.is_empty() {
    //         ""
    //     } else {
    //         &self.ring[0]
    //     };
    //     let msg = format!("Hola soy {}, vos sos {} y la fecha es {}\n", self.ip, neighbor_ip, Local::now()); 
    //     let stdin_thread = thread::spawn(move || {  //?
    //         //buscar otra manera / ver como hacer para manejar threads e instancias de self
    //         let stdin = io::stdin(); //input() para leer lo del server
    //         for line in stdin.lock().lines() {
    //             let input = line.unwrap();
    //             let mut socket = TcpStream::connect("127.0.0.1:7879").unwrap();
    //             let _ = socket.write(msg.as_bytes());
    //             let _ = socket.write(input.as_bytes());
    //             // socket.flush().unwrap();
    //         }
    //     });
    //     for stream in listener.incoming() {
    //         let stream = stream.unwrap();
    //         self.add_ring(); //solo conecta
    //         self.handle_connection(stream); //lee lo del cliente
    //     }
    // }

    pub fn connect_node(& mut self) { //version limpia, no hay comunacion server/server
        // para gossip y para conectar con cliente
        self.write_nodes();

        let listener = TcpListener::bind(&self.ip).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.add_ring(); //solo conecta
            self.handle_connection(stream); //lee lo del cliente
        }
    }

}

