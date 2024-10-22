use std::net::TcpStream;
use eframe::egui;
use egui::{Color32, FontId, Id};
use image::GenericImageView;

use crate::{clientstuff::client_cql::Client, config::MAP_PATH, cql::my_errors::MyError, gui::painted::{airport::Airport, 
        airport_coordinates::{get_airport_coordinates, get_origin_airport_of, get_return_airport_of}, 
        draw_flights::{draw_incoming_flights, draw_outgoing_flights}, 
        draw_lines::draw_lines_between_airports}};

pub struct MapApp {
    airports: Vec<Airport>,
    map_texture: Option<egui::TextureHandle>,
    client: Client,
    stream_client: Option<TcpStream>,
}

impl MapApp {
    pub fn new(port: String) -> Self {
        let airports = get_airport_coordinates();
        let map_texture = None;
        let client = Client::initialize(port);
        let stream_client = None;
        Self {
            airports,
            map_texture,
            client,
            stream_client,
        }
    }

    pub fn set_connection(&mut self) {
        if let Ok(stream) = self.client.connect() {
            self.stream_client = Some(stream);
        } else {
            //Cambiar de nodo?
            self.stream_client = None;
        }
    }

    pub fn load_map_texture(&mut self, ctx: &egui::Context) {
        // Cargar la imagen del mapa desde un archivo
        let map_image = image::open(MAP_PATH).expect("Failed to load map image");
        let (width, height) = map_image.dimensions();
        let image_rgba = map_image.to_rgba8();
        let texture = ctx.load_texture("map", egui::ColorImage::from_rgba_unmultiplied([width.try_into().unwrap(), height.try_into().unwrap()], &image_rgba), Default::default());
        self.map_texture = Some(texture)
    }
}

impl eframe::App for MapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Mapa de Aeropuertos");

            if self.map_texture.is_none() {
                self.load_map_texture(ctx);
            }

            if let Some(texture) = &self.map_texture {
                // Mostrar el mapa
                ui.image(texture);

                // Dibujar los aeropuertos
                for airport in &self.airports {
                    //draw_airport(ui, texture, airport);
                    let (lat, lon) = airport.coordinates;
                    let [width, height] = texture.size();
                    let screen_x = (lon + 180.0) * (width as f32 / 360.0);
                    let screen_y = (90.0 - lat) * (height as f32 / 180.0);
                
                    let radius_click = 15.0; // Radio del botón
                    let radius_paint = 5.0;

                    // Marcar el aeropuesto
                    ui.painter().circle_filled(egui::pos2(screen_x, screen_y), radius_paint, egui::Color32::RED);

                    // Posicionar el nombre
                    let name_pos = egui::pos2(screen_x + radius_paint + 5.0, screen_y); // Ajusta la posición según el radio
                    ui.painter().text(name_pos, egui::Align2::LEFT_CENTER, &airport.name, FontId::default(), Color32::WHITE);

                    // Posicionar el botón
                    let button_rect = egui::Rect::from_center_size(
                        egui::pos2(screen_x, screen_y), 
                        egui::vec2(radius_click * 2.0, radius_click * 2.0)
                    );

                    let date_selected = "".to_string();
                    // Crear un botón en la posición del aeropuerto
                    if ui.interact(button_rect, Id::new(&airport.name), egui::Sense::click()).clicked() {
                        println!("¡Aeropuerto {} presionado!", airport.name);

                        if let Ok(origin_flights) = airport.get_origin_flights_from(&self.stream_client, &date_selected) {
                            println!("origin_flights: {:?}", origin_flights);

                            let return_airports = get_return_airport_of(&origin_flights);

                            if let Ok(return_flights) = airport.get_flights_that_return_to(&self.stream_client, &date_selected) {

                                let origin_airports = get_origin_airport_of(&return_flights);
                                draw_lines_between_airports(&airport, &origin_airports);
                                draw_lines_between_airports(&airport, &return_airports);

                                draw_outgoing_flights(&origin_flights,&airport, &return_airports);
                                draw_incoming_flights(&return_flights, &origin_airports, &airport);

                            }
                        } else if let Err(origin_flights) = airport.get_origin_flights_from(&self.stream_client, &date_selected) {
                            println!("Error: {}", origin_flights);
                        }
                    }
                    
                }
            }
        });
    }
}
