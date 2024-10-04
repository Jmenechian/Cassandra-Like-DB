use aerolineas_rusticas::nodo2;
use aerolineas_rusticas::nodo2::*;
use std::env;
use std::fs::File;
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
    let ports_file = &args[1];

    nodo2::initialize(ports_file);
}
