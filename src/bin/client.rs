use aerolineas_rusticas::clientstuff::client_cql::Client;
use std::env;
use std::io::Result;
use std::process::exit;

/// Binario que va a correr el cliente que se conectará al servidor.
/// Posibilita realizar consultas CQL a las Tablas de Estado y las Tablas de Seguimiento de vuelos en curso.
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
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
