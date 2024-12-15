use crate::{
    config::{self, AIRPORTS_PATH},
    gui::connection_server::ConnectionServer,
};
use std::{collections::HashMap, f32::consts::PI, fs::read_to_string, sync::mpsc::Receiver};

use super::{airport::Airport, flight::Flight, queries::get_query_select_in_simulator};

/// Se devuelve un Hash de los aeropuestos del archivo ubicado en AIRPORTS_PATH.
/// La clave será nombre de cada aeropuerto.
pub fn get_airport_coordinates() -> HashMap<String, Airport> {
    let mut airports = HashMap::new();
    let file_lines = match read_to_string(AIRPORTS_PATH) {
        Ok(l) => l,
        Err(_) => "".to_string(),
    };
    for (index, line) in file_lines.lines().enumerate() {
        if index == 0 {
            continue;
        }
        let airport_data: Vec<&str> = line.split(",").collect();
        let airport_name = airport_data[0].to_string();
        let longitude: f32 = airport_data[2].parse::<f32>().unwrap_or(0.0);
        let latitude: f32 = airport_data[3].parse::<f32>().unwrap_or(0.0);
        let airport: Airport = Airport::new(airport_name.clone(), (longitude, latitude));
        airports.insert(airport_name, airport);
    }
    airports
}

/// Dado un vector de vuelos se obtiene un vector de aeropuestos que representan el origen de esos vuelos.
pub fn get_origin_airport_of(
    flights: &Vec<Flight>,
    airports: &HashMap<String, Airport>,
) -> Vec<Airport> {
    let mut origin_airports: Vec<Airport> = vec![];
    for flight in flights {
        if let Some(airport_aux) = airports.get(&flight.origin_airport.name) {
            origin_airports.push(airport_aux.clone());
        }
    }
    origin_airports
}

/// Dado un vector de vuelos se obtiene un vector de aeropuestos que representan el destino de esos vuelos.
pub fn get_return_airport_of(
    flights: &Vec<Flight>,
    airports: &HashMap<String, Airport>,
) -> Vec<Airport> {
    let mut return_airports: Vec<Airport> = vec![];
    for flight in flights {
        if let Some(airport_aux) = airports.get(&flight.return_airport.name) {
            return_airports.push(airport_aux.clone());
        }
    }
    return_airports
}

/// Dada una coordenada (x,y) se posiciona en el mapa, dependiendo de la texture de la imagen del mapa.
pub fn get_coordinate_positioning_on_map(
    (lon, lat): (f32, f32),
    [width, height]: [usize; 2],
) -> (f32, f32) {
    let screen_x = (lon + 180.0) * (width as f32 / 360.0);
    let screen_y = (90.0 - lat) * (height as f32 / 180.0);

    (screen_x, screen_y)
}

/// Dado dos coordenadas (x, y) se obtiene la posición media de estas.
pub fn obtain_middle_position(
    screen_x_o: f32,
    screen_x_d: f32,
    screen_y_o: f32,
    screen_y_d: f32,
) -> (f32, f32) {
    let mid_x = (screen_x_o + screen_x_d) / 2.0;
    let mid_y = (screen_y_o + screen_y_d) / 2.0;
    (mid_x, mid_y)
}

/// Dado dos coordenadas (x,y) se obtiene el ángulo que forman, expresado en grados.
/// Se mide desde el eje x positivo en el sentido de las agulas del reloj.
pub fn get_angle(screen_x_o: f32, screen_x_d: f32, screen_y_o: f32, screen_y_d: f32) -> f32 {
    let delta_x = screen_x_d - screen_x_o;
    let delta_y = screen_y_d - screen_y_o;
    let angle_radians = delta_y.atan2(delta_x);

    let angle_degrees = angle_radians * 180.0 / PI;

    // Ajustamos el angulo para que este en el rango 0 (incluido) a 360 (sin incluir) grados
    if angle_degrees < 0.0 {
        return angle_degrees + 360.0;
    }
    angle_degrees
}

/// Se busca si el vuelo está en la tabla del simulador.
/// En caso de que este, se devuelven las coordenadas que se obtengan de la tabla.
/// En caso contrario, se devulve los valores de la mitad del recorrido que se pasan por parametro.
pub fn integrate_simulator(
    flight: &Flight,
    (mid_x, mid_y): (f32, f32),
    connection_server: &ConnectionServer,
    [width, height]: [usize; 2],
) -> (f32, f32) {
    let query = get_query_select_in_simulator(
        flight.departure_date.clone(),
        flight.origin_airport.name.clone(),
        flight.number.clone(),
    );
    let channel: &Receiver<Vec<String>> = connection_server.get_channel_to_receive();
    println!("Query: {:?}", query);
    let _ = connection_server.make_query(query, config::ONE);
    let result = channel
        .recv()
        .unwrap_or(vec![mid_x.to_string(), mid_y.to_string()]);

    let flight_in_simulator: Vec<&str> = result[0].split(",").collect();
    if flight_in_simulator.len() == 10 {
        // [flight , airport , destination , status , date , speed , latitude , longitude , height, timestamp]
        if let (Ok(x), Ok(y)) = (
            flight_in_simulator[7].parse::<f32>(),
            flight_in_simulator[6].parse::<f32>(),
        ) {
            return get_coordinate_positioning_on_map((x, y), [width, height]);
        }
    }

    (mid_x, mid_y)
}
