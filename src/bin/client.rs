use aerolineas_rusticas::clientstuff::client_cql::Client;
use std::env;
use std::io::Result;
use std::process::exit;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo_de_puertos>", args[0]);
        exit(1);
    }
    let port = args[1].to_string();

    let client = Client::initialize(port);

    println!("Cliente: {:?}", client);

    if let Err(e) = client.handle_connection() {
        //solo manda no escucha
        println!("Error: {:?}", e);
    }
    Ok(())
}
