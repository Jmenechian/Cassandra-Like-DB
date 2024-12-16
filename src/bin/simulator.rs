use aerolineas_rusticas::errors::my_errors::MyError;
use aerolineas_rusticas::simulators::flight_simulator::FlightSimulator;
use std::env;
use std::net::IpAddr;
use std::process::exit;

/// Ejecución del simulador de vuelos.
/// Se crean vuelos en diferentes momentos del tiempo y se actualizan los valores de sus variables, imitando el comportamiento de un
/// avión que despega en un aeropuerto de origen, llega a velocidad crucero y aterriza en el aeropuerto de destino.
fn main() -> Result<(), MyError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(1);
    }

    let ip_server = args[1].to_string();
    let ip_server_addr: IpAddr = match ip_server.parse() {
        Ok(ip) => ip,
        Err(_) => return Err(MyError::Error("No puse obtener el ip".to_string())),
    };

    let port = args[2].to_string();
    let _port: u16 = match port.parse() {
        Ok(ip) => ip,
        Err(_) => return Err(MyError::Error("No puse obtener el puerto".to_string())),
    };

    if let Err(error) = FlightSimulator::run(ip_server_addr, _port) {
        println!("{}", error);
    }
    Ok(())
}
