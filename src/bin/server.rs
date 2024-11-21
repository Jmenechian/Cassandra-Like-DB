// Binario que va a correr cada nodo del servidor

use aerolineas_rusticas::serverstuff::node_initializer::node_initialize;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(1);
    }
    let ports_file = args[1].to_string();

    match node_initialize(ports_file) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{:?}", e);
        }
    };
}
