use aerolineas_rusticas::nodo2::Node;
use aerolineas_rusticas::nodo2;
// use aerolineas_rusticas::node::Node;
use aerolineas_rusticas::query::Query;
use aerolineas_rusticas::server_message::ServerMessage;
use std::env;
use std::fs::File;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo_de_puertos>", args[0]);
        std::process::exit(1);
    }
    let ports_file = args[1].to_string();

    nodo2::initialize(ports_file);
}

// fn main() {
//     let args: Vec<String> = env::args().collect();

//     if args.len() > 1 && args[1] == "Fix" { //cargo run -- Fix limpia nodes.txt
//         fs::write("nodes.txt", "").expect("Failed to clear nodes.txt");
//         return;
//     }

//     let mut node = Node::new(&args[1]);
//     let node_arc = Arc::new(Mutex::new(node));

//     match node.clone().connect_node() {
//         Ok(()) => {},
//         Err(e) => println!("Error: {}", e),
//     }
// }