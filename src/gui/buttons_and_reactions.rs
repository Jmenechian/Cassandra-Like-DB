use egui::Color32;

use crate::config::{DELETE, INSERT, UPDATE};

use super::queries::{
    get_destination_delete_query, get_destination_insert_query, get_destination_update_query,
    get_origin_delete_query, get_origin_insert_query, get_origin_update_query,
};

/// Devuelve las queries sobre las tablas de origen y destino dependiendo del query_type.
pub fn match_and_get_queries(
    query_type: i8,
    pressed_insert_or_update: bool,
    pressed_delete: bool,
    (origin_airport, destination_airport, date, flight, status): (
        &String,
        &String,
        &String,
        &String,
        &String,
    ),
) -> (String, String) {
    let mut query_origin = "".to_string();
    let mut query_destination = "".to_string();

    if query_type == INSERT && pressed_insert_or_update {
        query_origin =
            get_origin_insert_query(origin_airport, destination_airport, date, flight, status);
        query_destination =
            get_destination_insert_query(origin_airport, destination_airport, date, flight, status);
    }

    if query_type == UPDATE && pressed_insert_or_update {
        query_origin = get_origin_update_query(origin_airport, date, flight, status);
        query_destination = get_destination_update_query(destination_airport, date, flight, status);
    }

    if query_type == DELETE && pressed_delete {
        query_origin = get_origin_delete_query(origin_airport, date, flight);
        query_destination = get_destination_delete_query(destination_airport, date, flight);
    }

    (query_origin, query_destination)
}

/// Devuelve true si el botón de insert puede ser presionado, en caso contrario: false.
pub fn insert_or_update_button_can_be_pressed(
    (origin_airport, destination_airport, date, flight, status): (
        &String,
        &String,
        &String,
        &String,
        &String,
    ),
) -> bool {
    !origin_airport.is_empty()
        && !destination_airport.is_empty()
        && !date.is_empty()
        && !flight.is_empty()
        && !status.is_empty()
}

/// Devuelve true si los botones de de update y delete pueden ser presionado, en caso contrario: false.
pub fn delete_button_can_be_pressed(
    (origin_airport, destination_airport, date, flight, _status): (
        &String,
        &String,
        &String,
        &String,
        &String,
    ),
) -> bool {
    !date.is_empty()
        && !origin_airport.is_empty()
        && !flight.is_empty()
        && !destination_airport.is_empty()
}

/// Devulve color verde si se envía true, en caso contrario: color rojo.
pub fn get_color_button(condition: bool) -> Color32 {
    if condition {
        Color32::from_rgb(0, 128, 0) // Green when fields are filled
    } else {
        Color32::from_rgb(255, 0, 0) // Red when fields are empty
    }
}
