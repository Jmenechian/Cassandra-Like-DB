// Binario que va a correr cada nodo del servidor

use aerolineas_rusticas::nodo;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo_de_puertos>", args[0]);
        exit(1);
    }
    println!("args: {:?}", args);
    let ports_file = args[1].to_string();

    match nodo::initialize(ports_file) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{:?}", e);
        }
    };
}
