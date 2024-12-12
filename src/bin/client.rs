use aerolineas_rusticas::clientstuff::client_cql::Client;
use std::env;
use std::process::exit;

/// Binario que va a correr el cliente que se conectará al servidor.
/// Posibilita realizar consultas CQL a las Tablas de Estado y las Tablas de Seguimiento de vuelos en curso.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(1);
    }
    let port = args[1].to_string();

    if let Ok(client) = Client::initialize(port) {
        println!("Cliente: {:?}", client);

        if let Err(e) = client.handle_connection() {
            //solo manda no escucha
            println!("Error: {:?}", e);
        }
    }
}
