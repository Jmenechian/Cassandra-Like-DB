use eframe::egui;
use egui::{Color32, Context, FontId, Id, Painter, Rect, Stroke, TextureId};
use image::GenericImageView;
use std::{collections::HashMap, net::TcpStream};

use crate::{
    clientstuff::client_cql::Client,
    config::{MAP_PATH, PLANE_PATH, PLANE_SIZE, RADIUS_CLICK_AIRPORT, RADIUS_PAINT_AIRPORT},
    gui::painted::{
        airport::Airport,
        airport_coordinates::{
            get_airport_coordinates, get_angle, get_coordinate_positioning_on_map,
            obtain_middle_position,
        },
        flight::Flight,
    },
};

/// Estructura que representa la interfaz de usuario.
/// Mapa con aeropuestos y vuelos que conectan a los aeropuestos.
pub struct MapApp {
    airports: HashMap<String, Airport>,
    map_texture: Option<egui::TextureHandle>,
    plane_texture: Option<egui::TextureHandle>,
    client: Client,
    stream_client: Option<TcpStream>,
    connections: Vec<(Airport, Airport, Flight)>,
}

impl MapApp {
    /// Se inicializan los valores a usar en la interfaz.
    pub fn new(port: String) -> Self {
        let airports = get_airport_coordinates();
        let map_texture = None;
        let plane_texture = None;
        let client = Client::initialize(port);
        let stream_client = None;
        let connections = vec![];
        Self {
            airports,
            map_texture,
            plane_texture,
            client,
            stream_client,
            connections,
        }
    }

    /// Establece la conección con el nodo indicado.
    fn set_connection_server(&mut self) {
        if let Ok(stream) = self.client.connect() {
            self.stream_client = Some(stream);
        } else {
            //Cambiar de nodo?
            self.stream_client = None;
        }
    }

    /// Carga la imagen del avión desde un archivo.
    fn load_plane_texture(&mut self, ctx: &egui::Context) {
        let plane_image = image::open(PLANE_PATH).expect("Failed to load plane image");
        let (width, height) = plane_image.dimensions();
        let image_rgba = plane_image.to_rgba8();
        let texture = ctx.load_texture(
            "plane",
            egui::ColorImage::from_rgba_unmultiplied(
                [width.try_into().unwrap(), height.try_into().unwrap()],
                &image_rgba,
            ),
            Default::default(),
        );
        self.plane_texture = Some(texture);
    }

    /// Carga la imagen del mapa desde un archivo.
    fn load_map_texture(&mut self, ctx: &egui::Context) {
        let map_image = image::open(MAP_PATH).expect("Failed to load map image");
        let (width, height) = map_image.dimensions();
        let image_rgba = map_image.to_rgba8();
        let texture = ctx.load_texture(
            "map",
            egui::ColorImage::from_rgba_unmultiplied(
                [width.try_into().unwrap(), height.try_into().unwrap()],
                &image_rgba,
            ),
            Default::default(),
        );
        self.map_texture = Some(texture)
    }

    /// Agrega una conexión entre aeropuestos en el mapa. Esto permite que se muestre en cada update del mapa las lineas que unen estos aerpuestos.
    fn add_connection(&mut self, origin: Airport, destination: Airport, flight: Flight) {
        self.connections.push((origin, destination, flight));
    }

    /// Se resetean las conecciones entre aeropuesto. Se desconectan todos los aeropuestos.
    fn reset_connection(&mut self) {
        self.connections = vec![];
    }

    fn initialize_fields(&mut self, ctx: &Context) {
        if self.map_texture.is_none() {
            self.load_map_texture(ctx);
        }

        if self.plane_texture.is_none() {
            self.load_plane_texture(ctx);
        }

        if self.stream_client.is_none() {
            self.set_connection_server();
        }
    }

    fn draw_airports(
        &self,
        painter: &Painter,
        screen_x: f32,
        screen_y: f32,
        airport_name: &String,
    ) -> Rect {
        // Marcar el aeropuesto
        painter.circle_filled(
            egui::pos2(screen_x, screen_y),
            RADIUS_PAINT_AIRPORT,
            egui::Color32::RED,
        );

        // Posicionar el nombre
        let name_pos = egui::pos2(screen_x + RADIUS_PAINT_AIRPORT + 5.0, screen_y); // Ajusta la posición según el radio
        painter.text(
            name_pos,
            egui::Align2::LEFT_CENTER,
            airport_name,
            FontId::default(),
            Color32::WHITE,
        );

        // Posicionar el botón
        Rect::from_center_size(
            egui::pos2(screen_x, screen_y),
            egui::vec2(RADIUS_CLICK_AIRPORT * 2.0, RADIUS_CLICK_AIRPORT * 2.0),
        )
    }

    fn draw_line(
        &self,
        painter: &Painter,
        screen_x_o: f32,
        screen_y_o: f32,
        screen_x_d: f32,
        screen_y_d: f32,
    ) {
        painter.line_segment(
            [
                egui::pos2(screen_x_o, screen_y_o),
                egui::pos2(screen_x_d, screen_y_d),
            ],
            Stroke::new(5.0, Color32::GRAY), // Color y grosor de la línea
        );
    }

    fn draw_plane(
        &self,
        painter: &Painter,
        mid_x: f32,
        mid_y: f32,
        plane_texture_id: TextureId,
        flight_name: &String,
        _angle: f32,
    ) {
        let plane_rect = egui::Rect::from_center_size(
            egui::pos2(mid_x, mid_y),
            egui::vec2(PLANE_SIZE, PLANE_SIZE),
        );

        let uv_rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        //sprite imagenes

        painter.image(plane_texture_id, plane_rect, uv_rect, Color32::WHITE);

        // Posicionar el nombre
        let name_pos = egui::pos2(mid_x + 30.0, mid_y); // Ajusta la posicion segun la imagen del avion
        painter.text(
            name_pos,
            egui::Align2::LEFT_CENTER,
            flight_name,
            FontId::default(),
            Color32::WHITE,
        );
    }
}

impl eframe::App for MapApp {
    /// Se llama cada vez que es necesario volver a pintar la interfaz de usuario, lo que puede ocurrir muchas veces por segundo.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Mapa de Aeropuertos");

            self.initialize_fields(ctx);

            let texture = self.map_texture.clone();
            if let Some(texture) = &texture {
                // Mostrar el mapa
                ui.image(texture);
                let painter = ui.painter();

                // Dibujar los aeropuertos
                let airports = self.airports.clone();
                for airport in airports.values() {
                    let (screen_x, screen_y) =
                        get_coordinate_positioning_on_map(airport.coordinates, texture.size());

                    let button_rect =
                        self.draw_airports(painter, screen_x, screen_y, &airport.name);

                    let date_selected = "24-12-12".to_string();
                    // Crear un botón en la posición del aeropuerto
                    if ui
                        .interact(button_rect, Id::new(&airport.name), egui::Sense::click())
                        .clicked()
                    {
                        println!("¡Aeropuerto {} presionado!", airport.name);
                        self.reset_connection();

                        if let (Ok(origin_flights), Ok(return_flights)) = (
                            airport.get_origin_flights_from(
                                &mut self.stream_client,
                                &date_selected,
                                &self.airports,
                            ),
                            airport.get_flights_that_return_to(
                                &mut self.stream_client,
                                &date_selected,
                                &self.airports,
                            ),
                        ) {
                            if origin_flights.is_empty() && return_flights.is_empty() {
                                continue;
                            }

                            for flight_r in return_flights {
                                //airport_o
                                let (screen_x_o, screen_y_o) = get_coordinate_positioning_on_map(
                                    flight_r.origin_airport.coordinates,
                                    texture.size(),
                                );
                                if (screen_x, screen_y) != (screen_x_o, screen_y_o) {
                                    self.add_connection(
                                        airport.clone(),
                                        flight_r.origin_airport.clone(),
                                        flight_r,
                                    );
                                }
                            }

                            for flight_o in origin_flights {
                                // airport_r
                                let (screen_x_r, screen_y_r) = get_coordinate_positioning_on_map(
                                    flight_o.return_airport.coordinates,
                                    texture.size(),
                                );
                                if (screen_x, screen_y) != (screen_x_r, screen_y_r) {
                                    self.add_connection(
                                        airport.clone(),
                                        flight_o.return_airport.clone(),
                                        flight_o,
                                    );
                                }
                            }
                        }
                    }
                }

                let mut last_o: &Airport = &Airport::default_new();
                let mut last_r: &Airport = &Airport::default_new();
                for (origin, destination, flight) in &self.connections {
                    let (screen_x_o, screen_y_o) =
                        get_coordinate_positioning_on_map(origin.coordinates, texture.size());
                    let (screen_x_d, screen_y_d) =
                        get_coordinate_positioning_on_map(destination.coordinates, texture.size());

                    self.draw_line(painter, screen_x_o, screen_y_o, screen_x_d, screen_y_d);

                    // Posición media para el avión
                    let (mut mid_x, mut mid_y) =
                        obtain_middle_position(screen_x_o, screen_x_d, screen_y_o, screen_y_d);

                    if last_o == origin && last_r == destination {
                        (mid_x, mid_y) = (mid_x + 25.0, mid_y + 25.0);
                    }

                    // Ángulo de rotación
                    let angle = get_angle(screen_x_o, screen_x_d, screen_y_o, screen_y_d);

                    // Dibujar el avión
                    let plane_texture = self.plane_texture.clone();
                    if let Some(plane_texture) = plane_texture {
                        self.draw_plane(
                            painter,
                            mid_x,
                            mid_y,
                            plane_texture.id(),
                            &flight.name,
                            angle,
                        );
                    }

                    last_o = origin;
                    last_r = destination;
                }
            }
        });
    }
}
