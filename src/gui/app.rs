use eframe::egui;
use egui::{Color32, Context, FontId, Id, Painter, Rect, Stroke, TextureHandle, TextureId, Ui};

use image::GenericImageView;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use crate::{
    clientstuff::client_cql::Client,
    config::*,
    errors::my_errors::MyError,
    gui::user::Role,
    gui::user::User,
    simulators::{
        airport::Airport,
        airport_coordinates::{
            get_airport_coordinates, get_angle, get_coordinate_positioning_on_map,
            obtain_middle_position,
        },
        flight::Flight,
    },
};

use super::connection_server::ConnectionServer;

/// Estructura que representa la interfaz de usuario.
/// Mapa con aeropuestos y vuelos que conectan a los aeropuestos.
pub struct MapApp {
    airports: HashMap<String, Airport>,
    map_texture: Option<egui::TextureHandle>,
    plane_texture: Option<egui::TextureHandle>,
    connection_server: ConnectionServer,
    connections: Vec<(Airport, Airport, Flight)>,
    date_selected: String,
    airport_selected: String,
    airport2_selected: String,
    status_selected: String,
    flight_selected: String,
    username: String,
    password: String,
    user: Option<User>,
    login_status: String,
}

impl MapApp {
    /// Se inicializan los valores a usar en la interfaz.
    pub fn new(port: String) -> Self {
        let airports = get_airport_coordinates();
        let map_texture = None;
        let plane_texture = None;
        let client = Client::initialize(port);
        let connection_server = ConnectionServer::initialize(client);
        let connections = vec![];

        //botones
        let airport_selected = "".to_string();
        let airport2_selected = "".to_string();
        let status_selected = "".to_string();
        let flight_selected = "".to_string();
        let date_selected = "".to_string();

        // usuario
        let username = "".to_string();
        let password = "".to_string();
        let user = None;
        let login_status = "Por favor ingresar usuario y contraseña".to_string();
        Self {
            airports,
            map_texture,
            plane_texture,
            connection_server,
            connections,

            //botones
            airport_selected,
            airport2_selected,
            status_selected,
            flight_selected,
            date_selected,

            //usuario
            username,
            password,
            user,
            login_status,
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
    fn add_all_connections(
        &mut self,
        airport: &Airport,
        texture: &TextureHandle,
        screen_x: f32,
        screen_y: f32,
    ) -> Result<(), MyError> {
        let rx: &Receiver<Vec<String>> = self.connection_server.get_channel_to_receive();

        self.connection_server
            .send_origin_and_return_flights(self.date_selected.clone(), airport.clone())?;

        let origin_flights = rx.recv().unwrap();
        let return_flights = rx.recv().unwrap();

        let origin_flights =
            airport.parse_origin_flights(&self.airports.clone(), origin_flights)?;
        let return_flights =
            airport.parse_return_flights(&self.airports.clone(), return_flights)?;

        if origin_flights.is_empty() && return_flights.is_empty() {
            return Ok(());
        }

        self.iter_and_add_connections("origin", return_flights, texture, (screen_x, screen_y));
        self.iter_and_add_connections("return", origin_flights, texture, (screen_x, screen_y));
        Ok(())
    }

    fn iter_and_add_connections(
        &mut self,
        connect_with: &str,
        flights: Vec<Flight>,
        texture: &TextureHandle,
        coordenates: (f32, f32),
    ) {
        for flight in flights {
            let (screen_x, screen_y) = match connect_with {
                "origin" => get_coordinate_positioning_on_map(
                    flight.origin_airport.coordinates,
                    texture.size(),
                ),
                "return" => get_coordinate_positioning_on_map(
                    flight.return_airport.coordinates,
                    texture.size(),
                ),
                _ => return,
            };

            if coordenates != (screen_x, screen_y) {
                self.add_connection(
                    flight.origin_airport.clone(),
                    flight.return_airport.clone(),
                    flight,
                );
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
                    &flight.number,
                    angle,
                );
            }

            last_o = origin;
            last_r = destination;
        }
    }

    /// Se dibuja la tabla con los detalles del avión.
    fn draw_flight_table(&mut self, ui: &mut Ui, texture: &TextureHandle) {
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
                        ui.label("");
                        ui.end_row();
                        for (origin, destination, flight) in &self.connections.clone() {
                            ui.label(flight.number.clone());
                            ui.label(origin.name.clone());
                            ui.label(destination.name.clone());
                            ui.label(flight.status.to_str());
                            if let Some(ref user) = self.user {
                                if user.role == Role::User {
                                    ui.end_row();
                                    continue;
                                }
                            }
                            if ui.button("🗑️").clicked() {
                                let query_origen = format!(
                                    "DELETE FROM origin WHERE date={} AND airport={} AND flight={}",
                                    flight.departure_date, origin.name, flight.number
                                );
                                let query_destino = format!(
                                    "DELETE FROM destination WHERE date={} AND airport={} AND flight={}",
                                    flight.departure_date, destination.name, flight.number
                                );

                                if !query_origen.is_empty() && !query_destino.is_empty() {
                                    let rx: &Receiver<Vec<String>> = self.connection_server.get_channel_to_receive();

                                    let _ = self.connection_server.make_query(query_origen);
                                    let _ = rx.recv().unwrap();
                                    let _ = self.connection_server.make_query(query_destino);
                                    let _ = rx.recv().unwrap();

                                    if let Some(airport) = self.airports.clone().get(&self.airport_selected) {
                                        let (screen_x, screen_y) =
                                        get_coordinate_positioning_on_map(airport.coordinates, texture.size());
                                            self.reset_connection();
                                            let _ = self.add_all_connections(airport, texture, screen_x, screen_y);
                                    }
                                }
                            }
                            ui.end_row();
                        }
                    });
                ui.separator();
            }
        });
    }

    fn draw_other_box(&mut self, ui: &mut Ui, x: f32, y: f32, tag: &str, var_number: i8) {
        //caja de input con input keyboard
        let var = match var_number {
            0 => &mut self.airport_selected,
            2 => &mut self.airport2_selected,
            3 => &mut self.date_selected,
            _ => &mut self.flight_selected,
        };
        egui::Area::new(Id::new(tag))
            .fixed_pos(egui::pos2(x, y))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    let label = format!("Enter {}:", tag);
                    ui.label(label);
                    ui.text_edit_singleline(var);
                    *var = var.trim().to_string();
                });
            });
    }

    fn draw_status_box(&mut self, ui: &mut Ui, x: f32, y: f32, tag: &str) {
        egui::Area::new(Id::new(tag))
            .fixed_pos(egui::pos2(x, y))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    let label = format!("Enter {}:", tag);
                    ui.label(label);

                    egui::ComboBox::from_label("")
                        .selected_text(&self.status_selected)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.status_selected,
                                "scheduled".to_string(),
                                "scheduled",
                            )
                            .clicked();
                            ui.selectable_value(
                                &mut self.status_selected,
                                "airborne".to_string(),
                                "airborne",
                            )
                            .clicked();
                            ui.selectable_value(
                                &mut self.status_selected,
                                "delayed".to_string(),
                                "delayed",
                            )
                            .clicked();
                            ui.selectable_value(
                                &mut self.status_selected,
                                "cancelled".to_string(),
                                "cancelled",
                            )
                            .clicked();
                            ui.selectable_value(
                                &mut self.status_selected,
                                "boarding".to_string(),
                                "boarding",
                            )
                            .clicked();
                            ui.selectable_value(
                                &mut self.status_selected,
                                "landed".to_string(),
                                "landed",
                            )
                            .clicked();
                        });
                });
            });
    }
    fn draw_fixed_button(
        &mut self,
        ui: &mut Ui,
        x: f32,
        y: f32,
        tag: &str,
        var: i8,
        texture: &TextureHandle,
        color: Color32,
    ) {
        let insert_button = !self.airport_selected.is_empty()
            && !self.airport2_selected.is_empty()
            && !self.date_selected.is_empty()
            && !self.flight_selected.is_empty()
            && !self.status_selected.is_empty();
        let up_del_button = !self.date_selected.is_empty()
            && !self.airport_selected.is_empty()
            && !self.flight_selected.is_empty()
            && !self.airport2_selected.is_empty();
        egui::Area::new(Id::new(tag))
            .fixed_pos(egui::pos2(x, y))
            .show(ui.ctx(), |ui| {
                let mut query_origen = "".to_string();
                let mut query_destino = "".to_string();
                if ui
                    .add(egui::Button::new(tag).fill(color))
                    .clicked()
                {

                    if var == INSERT && insert_button{
                        query_origen = format!(
                            "INSERT INTO origin (airport, destination, date, flight, status) VALUES ('{}', '{}', '{}', '{}', '{}')",
                            self.airport_selected, self.airport2_selected, self.date_selected, self.flight_selected, self.status_selected
                        );
                        query_destino = format!(
                            "INSERT INTO destination (airport, origin, date, flight, status) VALUES ('{}', '{}', '{}', '{}', '{}')",
                            self.airport2_selected, self.airport_selected, self.date_selected, self.flight_selected, self.status_selected
                        );
                    }

                    if var == UPDATE && up_del_button {
                        query_origen = format!(
                            "UPDATE origin SET status = '{}' WHERE date = '{}' AND airport = '{}' AND flight = '{}'",
                            self.status_selected, self.date_selected, self.airport_selected, self.flight_selected
                        );
                        query_destino = format!(
                            "UPDATE destination SET status = '{}' WHERE date = '{}' AND airport = '{}' AND flight = '{}'",
                            self.status_selected, self.date_selected, self.airport2_selected, self.flight_selected
                        );
                    }

                    if var == DELETE && up_del_button{
                        query_origen = format!(
                            "DELETE FROM origin WHERE date={} AND airport={} AND flight={}",
                            self.date_selected, self.airport_selected, self.flight_selected
                        );
                        query_destino = format!(
                            "DELETE FROM destination WHERE date={} AND airport={} AND flight={}",
                            self.date_selected, self.airport2_selected, self.flight_selected
                        );
                    }
                    if !query_origen.is_empty() && !query_destino.is_empty() {
                        let rx: &Receiver<Vec<String>> = self.connection_server.get_channel_to_receive();

                        let _ = self.connection_server.make_query(query_origen);
                        let _ = rx.recv().unwrap();
                        let _ = self.connection_server.make_query(query_destino);
                        let _ = rx.recv().unwrap();
                        println!("Ok receive response query");

                        if let Some(airport) = self.airports.clone().get(&self.airport_selected){
                            let (screen_x, screen_y) =
                            get_coordinate_positioning_on_map(airport.coordinates, texture.size());
                                self.reset_connection();
                                let _ = self.add_all_connections(airport, texture, screen_x, screen_y);
                                }
                    }
                }
            });
    }

    fn super_function(&mut self, ui: &mut Ui, texture: &TextureHandle) {
        //
        let mut y = 660.0;
        let mut x = 10.0;

        if self.user.is_none() {
            return;
        }

        if let Some(ref user) = self.user {
            if user.role == Role::User {
                self.draw_other_box(ui, x, y, "date", DATE); //date
                return;
            }
        }

        self.draw_other_box(ui, x, y, "Origen", AIRPORT); //aeropuerto
        y = offset_y(y);
        self.draw_other_box(ui, x, y, "Destino", AIRPORT2); //aeropuerto2
        y = offset_y(y);
        self.draw_other_box(ui, x, y, "date", DATE); //date
        y = offset_y(y);
        self.draw_other_box(ui, x, y, "flight", FLIGHT); //flight
        y = offset_y(y);
        self.draw_status_box(ui, x, y, "status"); //status
        y = offset_y(y);

        let insert_button = !self.airport_selected.is_empty()
            && !self.airport2_selected.is_empty()
            && !self.date_selected.is_empty()
            && !self.flight_selected.is_empty()
            && !self.status_selected.is_empty();
        let up_del_button = !self.date_selected.is_empty()
            && !self.airport_selected.is_empty()
            && !self.flight_selected.is_empty()
            && !self.airport2_selected.is_empty();

        let insert_color = if insert_button {
            Color32::from_rgb(0, 128, 0) // Green when fields are filled
        } else {
            Color32::from_rgb(255, 0, 0) // Red when fields are empty
        };
        let up_del_color = if up_del_button {
            Color32::from_rgb(0, 128, 0) // Green when fields are filled
        } else {
            Color32::from_rgb(255, 0, 0) // Red when fields are empty
        };

        self.draw_fixed_button(ui, x, y, "INSERT", INSERT, texture, insert_color); //INSERT
        x = offset_x(x);
        self.draw_fixed_button(ui, x, y, "DELETE", DELETE, texture, up_del_color); //DELETE
        x = offset_x(x);
        self.draw_fixed_button(ui, x, y, "UPDATE", UPDATE, texture, up_del_color);
        //UPDATE
    }

    fn get_user(&mut self) {
        println!(
            "Attempting to login with username: {} and password: {}",
            self.username, self.password
        );

        match User::new(self.username.clone(), self.password.clone()) {
            Ok(user) => {
                println!("Login successful: {:?}", user);
                self.login_status = format!("Bienvenido {}!", user.username);
                self.user = Some(user);
            }
            Err(err) => {
                println!("Login failed: {:?}", err);
                self.login_status = "Usuario o contraseña incorrectos".to_string();
                self.user = None;
            }
        };
    }
}

impl eframe::App for MapApp {
    /// Se llama cada vez que es necesario volver a pintar la interfaz de usuario, lo que puede ocurrir muchas veces por segundo.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Dibujar login
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.add(egui::TextEdit::singleline(&mut self.username).hint_text("Enter username"));

                ui.label("Password:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Enter password"),
                );

                if ui.button("Login").clicked() {
                    self.get_user();
                }
                ui.label(&self.login_status);
            })
        });

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
                            self.airport_selected = airport.name.clone();
                            self.airport2_selected = "".to_string();
                            self.flight_selected = "".to_string();
                            self.reset_connection();

                            if !self.date_selected.is_empty() {
                                let _ =
                                    self.add_all_connections(airport, texture, screen_x, screen_y);
                            }
                        }
                    }

                    self.draw_connections_with_flight(texture, ui);

                    self.draw_flight_table(ui, texture);
                });

                self.super_function(ui, texture);
            }
        });
    }
}

fn offset_y(y: f32) -> f32 {
    y + 30.0
}

fn offset_x(x: f32) -> f32 {
    x + 100.0
}
