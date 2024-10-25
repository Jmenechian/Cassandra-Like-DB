use std::{collections::HashMap, fs::read_to_string, usize};
use crate::{config::AIRPORTS_PATH, gui::painted::{airport::Airport, flight::Flight}};

pub fn get_airport_coordinates() -> HashMap<String, Airport>{
    let mut airports = HashMap::new();
    for (index, line) in read_to_string(AIRPORTS_PATH).unwrap().lines().enumerate() {
        if index == 0 {
            continue;
        }
        let airport_data: Vec<&str> = line.split(",").collect();
        let airport_name = airport_data[0].to_string();
        let longitude: f32 = airport_data[2].parse::<f32>().unwrap();
        let latitude: f32 = airport_data[3].parse::<f32>().unwrap();
        let airport: Airport = Airport::new(airport_name.clone(), (latitude, longitude));
        airports.insert(airport_name, airport);
    }
    airports
}

pub fn get_origin_airport_of(flights: &Vec<Flight>, airports: &HashMap<String, Airport>) -> Vec<Airport>{
    let mut origin_airports: Vec<Airport> = vec![];
    for flight in flights {
        if let Some(airport_aux) = airports.get(&flight.origin_airport.name) {
            origin_airports.push(airport_aux.clone());
        }
    }
    origin_airports
}

pub fn get_return_airport_of(flights: &Vec<Flight>, airports: &HashMap<String, Airport>) -> Vec<Airport> {
    let mut return_airports: Vec<Airport> = vec![];
    for flight in flights {
        if let Some(airport_aux) = airports.get(&flight.return_airport.name) {
            return_airports.push(airport_aux.clone());
        }
    }
    return_airports
}

pub fn get_coordinate_positioning_on_map((lat, lon): (f32, f32), [width, height]: [usize; 2]) -> (f32, f32) {    
    let screen_x = (lon + 180.0) * (width as f32 / 360.0);
    let screen_y = (90.0 - lat) * (height as f32 / 180.0);

    (screen_x, screen_y)
}

pub fn obtain_middle_position(screen_x_o: f32, screen_x_d: f32, screen_y_o: f32, screen_y_d: f32) -> (f32, f32) {
    let mid_x = (screen_x_o + screen_x_d) / 2.0;
    let mid_y = (screen_y_o + screen_y_d) / 2.0;
    (mid_x, mid_y)
}

pub fn get_angle(screen_x_o: f32, screen_x_d: f32, screen_y_o: f32, screen_y_d: f32) -> f32 {
    let delta_x = screen_x_d - screen_x_o;
    let delta_y = screen_y_d - screen_y_o;
    delta_y.atan2(delta_x) // ángulo en radianes
}
