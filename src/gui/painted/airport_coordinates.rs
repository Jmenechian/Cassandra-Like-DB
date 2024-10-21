use super::airport::Airport;
use std::fs::read_to_string;
use crate::config::FILE_AIRPORT;

pub fn get_airport_coordinates() -> Vec<Airport> {
    let mut airports: Vec<Airport> = vec![];
    for (index, line) in read_to_string(FILE_AIRPORT).unwrap().lines().enumerate() {
        if index == 0 {
            continue;
        }
        let airport_data: Vec<&str> = line.split(",").collect();
        let airport_name = airport_data[0].to_string();
        let longitude: f32 = airport_data[2].parse::<f32>().unwrap();
        let latitude: f32 = airport_data[3].parse::<f32>().unwrap();
        let airport: Airport = Airport { name: airport_name, coordinates: (latitude, longitude) };
        airports.push(airport);
    }
    airports
}
