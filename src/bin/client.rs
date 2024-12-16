use aerolineas_rusticas::clientstuff::client_cql::Client;
use aerolineas_rusticas::errors::my_errors::MyError;
use std::env;
use std::process::exit;
use std::net::IpAddr;

/// Binario que va a correr el cliente que se conectará al servidor.
/// Posibilita realizar consultas CQL a las Tablas de Estado y las Tablas de Seguimiento de vuelos en curso.
fn main() -> Result<(), MyError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        exit(1);
    }

    let ip_node = args[1].to_string();
    let ip_node_addr: IpAddr = match ip_node.parse() {
        Ok(ip) => ip,
        Err(_) => return Err(MyError::Error("No puse obtener el ip".to_string())),
    };

    let port = args[2].to_string();
    let _port: u16 = match port.parse() {
        Ok(ip) => ip,
        Err(_) => return Err(MyError::Error("No puse obtener el puerto".to_string())),
    };

    let client = Client::initialize(ip_node_addr, _port);

    println!("Cliente: {:?}", client);

    if let Err(e) = client.handle_connection() {
        //solo manda no escucha
        println!("Error: {:?}", e);
    }
    Ok(())
}
