use aerolineas_rusticas::nodo;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo_de_puertos>", args[0]);
        std::process::exit(1);
    }
    let ports_file = args[1].to_string();

    nodo::initialize(ports_file);
}
