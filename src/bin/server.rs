use aerolineas_rusticas::errors::my_errors::MyError;
use aerolineas_rusticas::serverstuff::node_initializer::node_initialize;
use std::env;
use std::net::IpAddr;
use std::process::exit;

/// Binario que va a correr cada nodo del servidor.
fn main() -> Result<(), MyError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(1);
    }

    let ip_node: String = args[1].to_string();
    let ip_node_addr: IpAddr = match ip_node.parse() {
        Ok(ip) => ip,
        Err(_) => {
            return Err(MyError::Error(format!(
                "No pude obtener mi ip: {:?}",
                args[1]
            )))
        }
    };

    match node_initialize(ip_node_addr) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{:?}", e);
        }
    };
    Ok(())
}
