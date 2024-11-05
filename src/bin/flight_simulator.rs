use aerolineas_rusticas::simulators::flight_simulator::FlightSimulator;
use std::io::Result;

/// Ejecución del simulador de vuelos.
/// Se crean vuelos en diferentes momentos del tiempo y se actualizan los valores de sus variables, imitando el comportamiento de un
/// avión que despega en un aeropuerto de origen, llega a velocidad crucero y aterriza en el aeropuerto de destino.
fn main() -> Result<()> {
    let mut f_simulator = FlightSimulator::new();

    if let Err(e) = f_simulator.run() {
        println!("Error: {:?}", e);
    }

    Ok(())
}
