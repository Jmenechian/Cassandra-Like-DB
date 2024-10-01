use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

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
    pub neighbors: Vec<Node>, //RF
    pub ring: Vec<Node>, //Gossip
}

impl Node {

    pub fn new() -> Self {
        Self {
            status: StatusNode::Desconnected,
            ip: "127.0.0.1:7878".to_string(),
            file_path: "".to_string(),
            neighbors: Vec::new(),
            ring: Vec::new(),
        }
    }

    pub fn add_neighbor(&mut self, neighbor: Node) {
        self.neighbors.push(neighbor);
    }

    pub fn add_ring(&mut self, ring: Node) {
        self.ring.push(ring);
    }

    pub fn connect_node(&self) {
        // para gossip y para conectar con cliente
        let listener = TcpListener::bind(&self.ip).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("{:?}",stream);

            self.handle_connection(stream);
        }
    }

    pub fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
    
        println!("Request: {http_request:#?}");
    }

}

