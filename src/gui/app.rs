use eframe::egui;
use egui::{Color32, Context, FontId, Id, Painter, Rect, Stroke, TextureHandle, TextureId, Ui};

use image::GenericImageView;
use std::{collections::HashMap, net::TcpStream, thread};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc, Mutex};

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
    date_selected: String,
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
        let date_selected = "".to_string();
        Self {
            airports,
            map_texture,
            plane_texture,
            client,
            stream_client,
            connections,
            date_selected,
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
        let width: usize = width.try_into().unwrap_or(0);
        let height: usize = height.try_into().unwrap_or(0);
        let texture = ctx.load_texture(
            "plane",
            egui::ColorImage::from_rgba_unmultiplied([width, height], &image_rgba),
            Default::default(),
        );
        self.plane_texture = Some(texture);
    }

    /// Carga la imagen del mapa desde un archivo.
    fn load_map_texture(&mut self, ctx: &egui::Context) {
        let map_image = image::open(MAP_PATH).expect("Failed to load map image");
        let (width, height) = map_image.dimensions();
        let image_rgba = map_image.to_rgba8();
        let width: usize = width.try_into().unwrap_or(0);
        let height: usize = height.try_into().unwrap_or(0);
        println!("width: {}", width);
        println!("height: {}", height);

        let texture = ctx.load_texture(
            "map",
            egui::ColorImage::from_rgba_unmultiplied([width, height], &image_rgba),
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

    /// Se agrega las conecciones que tiene el aeropuesto con los demás por los aviones que salen o regresan de este.
    fn add_all_connections(&mut self, airport: &Airport, texture: &TextureHandle, screen_x: f32, screen_y: f32) {
        
        let (tx, rx): (Sender<Vec<Flight>>, Receiver<Vec<Flight>>) = mpsc::channel();
        let thread_tx: Sender<Vec<Flight>> = tx.clone();

        let stream_client: Arc<Mutex<Option<TcpStream>>> = match &self.stream_client {
            Some(stream) => Arc::new(Mutex::new(Some(stream.try_clone().expect("Failed to clone TcpStream")))),
            None => return,
        };  
        let date_selected: Arc<String> = Arc::new(self.date_selected.clone());
        let airports: Arc<HashMap<String, Airport>> = Arc::new(self.airports.clone());
        let airport = Arc::new(airport.clone());
        
        let _ = thread::spawn({
            let thread_tx = thread_tx.clone();
            let airport = Arc::clone(&airport); 
            
            move || {
                if let Some(stream) = stream_client.lock().unwrap().take() { {
                
                    if let (Ok(origin_flights), Ok(return_flights)) = (
                        airport.get_origin_flights_from(
                            &mut Some(stream.try_clone().unwrap()),
                            &date_selected,
                            &airports,
                        ),
                        airport.get_flights_that_return_to(
                            &mut Some(stream),
                            &date_selected,
                            &airports,
                        ),
                    ) {
                        thread_tx.send(origin_flights).unwrap();
                        thread_tx.send(return_flights).unwrap();
                    }
                }
        }}});
        
        let origin_flights = rx.recv().unwrap();
        let return_flights = rx.recv().unwrap();
        
        {
            if origin_flights.is_empty() && return_flights.is_empty() {
                return;
            }

            for flight_r in return_flights {
                //airport_o
                let (screen_x_o, screen_y_o) =
                    get_coordinate_positioning_on_map(
                        flight_r.origin_airport.coordinates,
                        texture.size(),
                    );
                if (screen_x, screen_y) != (screen_x_o, screen_y_o) {
                    self.add_connection(
                        flight_r.origin_airport.clone(),
                        flight_r.return_airport.clone(),
                        flight_r,
                    );
                }
                
            }

            for flight_o in origin_flights {
                // airport_r
                let (screen_x_r, screen_y_r) =
                    get_coordinate_positioning_on_map(
                        flight_o.return_airport.coordinates,
                        texture.size(),
                    );
                if (screen_x, screen_y) != (screen_x_r, screen_y_r) {
                    self.add_connection(
                        flight_o.origin_airport.clone(),
                        flight_o.return_airport.clone(),
                        flight_o,
                    );
                }
            }
        }
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

    /// Se dibuja las conecciones entre aeropuestos y su avion con el numero de avion.
    fn draw_connections_with_flight(&self, texture: &TextureHandle, ui: &mut Ui) {
        let mut last_o: &Airport = &Airport::default_new();
        let mut last_r: &Airport = &Airport::default_new();
        for (origin, destination, flight) in &self.connections {
            let (screen_x_o, screen_y_o) =
                get_coordinate_positioning_on_map(origin.coordinates, texture.size());
            let (screen_x_d, screen_y_d) =
                get_coordinate_positioning_on_map(destination.coordinates, texture.size());

            self.draw_line(ui.painter(), screen_x_o, screen_y_o, screen_x_d, screen_y_d);

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
                    ui.painter(),
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

    /// Se dibuja la tabla con los detalles del avión.
    fn draw_flight_table(&self, ui: &mut Ui) {
        // Tabla de vuelos
        // Usar un layout vertical para mantener el contenido en la parte superior
        ui.vertical(|ui| {
            ui.set_min_width(200.0); // Ancho mínimo para la tabla
                if !self.connections.is_empty() {
                    ui.separator();
                    ui.label("Flight Information:");
                    egui::Grid::new("flight_info_table")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Flight");
                            ui.label("Origin");
                            ui.label("Destination");
                            ui.label("Status");
                            ui.end_row();
                            for (origin, destination, flight) in &self.connections {
                                ui.label(flight.name.clone());
                                ui.label(origin.name.clone());
                                ui.label(destination.name.clone());
                                ui.label(flight.status.clone());
                                ui.end_row();
                            }
                    });
                    ui.separator();
                }
        });
    }

    /// Se dibuja el box para engresar la fecha.
    fn draw_date_box(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ..{
                ui.label("Enter date:");
                ui.text_edit_singleline(&mut self.date_selected);
                self.date_selected = self.date_selected.trim().to_string();
            }
        });
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
                ui.horizontal(|ui| {
                    // Mostrar el mapa
                    ui.image(texture);

                    // Dibujar los aeropuertos
                    let airports = self.airports.clone();
                    for airport in airports.values() {
                        let (screen_x, screen_y) =
                            get_coordinate_positioning_on_map(airport.coordinates, texture.size());

                        let button_rect =
                            self.draw_airports(ui.painter(), screen_x, screen_y, &airport.name);

                        // Botón en la posición del aeropuerto
                        if ui
                            .interact(button_rect, Id::new(&airport.name), egui::Sense::click())
                            .clicked()
                        {
                            println!("¡Aeropuerto {} presionado!", airport.name);

                            self.reset_connection();

                            if !self.date_selected.is_empty() {
                                // thread::spawn(move || {
                                    self.add_all_connections(airport, texture, screen_x, screen_y);
                                // });
                            }
                        }
                    }

                    self.draw_connections_with_flight(texture, ui);

                    self.draw_flight_table(ui);
                });

                self.draw_date_box(ui);
            }
        });

    }
}
