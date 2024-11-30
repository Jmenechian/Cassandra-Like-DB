use eframe::egui;
use egui::{Color32, Context, FontId, Id, Painter, Rect, Stroke, TextureHandle, TextureId, Ui};

use image::GenericImageView;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use crate::{
    clientstuff::client_cql::Client,
    config::{self, *},
    errors::my_errors::MyError,
    gui::user::{Role, User},
    simulators::{
        airport::Airport,
        coordinates::{
            get_airport_coordinates, get_angle, get_coordinate_positioning_on_map,
            integrate_simulator, obtain_middle_position,
        },
        flight::Flight,
    },
};

use super::{
    buttons_and_reactions::{
        delete_button_can_be_pressed, get_color_button, insert_or_update_button_can_be_pressed,
        match_and_get_queries,
    },
    connection_server::ConnectionServer,
    images::{get_all_angles_for_flight, get_texture_for_flight},
};

/// Estructura que representa la interfaz de usuario.
/// Log in:
///     - En caso de ser un usuario admin, puede modificar los vuelos, agregar nuevos y borrar exitentes.
///     - Caso contrario, sólo puede visualizar los vuelos.
/// Mapa con aeropuestos y vuelos que conectan a los aeropuestos.
pub struct MapApp {
    map_texture: Option<egui::TextureHandle>,
    plane_texture: HashMap<i32, Option<egui::TextureHandle>>,
    airports: HashMap<String, Airport>,
    connections: Vec<(Airport, Airport, Flight)>, // Conexiones entre aeropuestos por medio del vuelo indicado.

    connection_server: ConnectionServer,

    date_selected: String,
    selected_origin_airport: String,
    selected_destination_airport: String,
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
        let plane_texture = HashMap::new();
        let client = Client::initialize(port);
        let connection_server = ConnectionServer::initialize(client);
        let connections = vec![];

        //botones
        let selected_origin_airport = "".to_string();
        let selected_destination_airport = "".to_string();
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
            selected_origin_airport,
            selected_destination_airport,
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
        for angle in get_all_angles_for_flight() {
            let path = PLANE_PATH.replace("{}", &angle.to_string());
            let plane_image = image::open(path).expect("Failed to load plane image");
            let (width, height) = plane_image.dimensions();
            let image_rgba = plane_image.to_rgba8();
            let width: usize = width.try_into().unwrap_or(0);
            let height: usize = height.try_into().unwrap_or(0);
            let texture = ctx.load_texture(
                "plane",
                egui::ColorImage::from_rgba_unmultiplied([width, height], &image_rgba),
                Default::default(),
            );
            self.plane_texture.insert(angle, Some(texture));
        }
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

    /// Se resetean las conexiones entre aeropuesto. Se desconectan todos los aeropuestos.
    fn reset_connection(&mut self) {
        self.connections = vec![];
    }
    /// Se agrega las conexiones que tiene el aeropuesto con los demás por los aviones que salen o regresan de este.
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

        if self.plane_texture.is_empty() {
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
        x: f32,
        y: f32,
        plane_texture_id: TextureId,
        flight_name: &String,
        _angle: f32,
    ) {
        let plane_rect =
            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(PLANE_SIZE, PLANE_SIZE));

        let uv_rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        painter.image(plane_texture_id, plane_rect, uv_rect, Color32::WHITE);

        // Posicionar el nombre
        let name_pos = egui::pos2(x + 30.0, y); // Ajusta la posicion segun la imagen del avion
        painter.text(
            name_pos,
            egui::Align2::LEFT_CENTER,
            flight_name,
            FontId::default(),
            Color32::WHITE,
        );
    }

    /// Se dibuja las conexiones entre aeropuestos y su avion con el numero de avion.
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

            let (x, y) = integrate_simulator(
                flight,
                (mid_x, mid_y),
                &self.connection_server,
                texture.size(),
            );

            // Ángulo de rotación
            let angle = get_angle(screen_x_o, screen_x_d, screen_y_o, screen_y_d);

            // Dibujar el avión
            let plane_texture = self.plane_texture.clone();
            let texture = get_texture_for_flight(angle, plane_texture);

            if let Some(plane_texture) = texture {
                self.draw_plane(
                    ui.painter(),
                    x,
                    y,
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
                                let query_origin = format!(
                                    "DELETE FROM origin WHERE date={} AND airport={} AND flight={}",
                                    flight.departure_date, origin.name, flight.number
                                );
                                let query_destination = format!(
                                    "DELETE FROM destination WHERE date={} AND airport={} AND flight={}",
                                    flight.departure_date, destination.name, flight.number
                                );

                                if !query_origin.is_empty() && !query_destination.is_empty() {
                                    let rx: &Receiver<Vec<String>> = self.connection_server.get_channel_to_receive();

                                    let _ = self.connection_server.make_query(query_origin, config::QUORUM);
                                    let _ = rx.recv().unwrap();
                                    let _ = self.connection_server.make_query(query_destination, config::QUORUM);
                                    let _ = rx.recv().unwrap();

                                    if let Some(airport) = self.airports.clone().get(&self.selected_origin_airport) {
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
            0 => &mut self.selected_origin_airport,
            2 => &mut self.selected_destination_airport,
            3 => &mut self.date_selected,
            _ => &mut self.flight_selected,
        };
        egui::Area::new(Id::new(tag))
            .fixed_pos(egui::pos2(x, y))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    let label = format!("Enter {}:", tag);
                    ui.label(egui::RichText::new(label).heading());
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
                    ui.label(egui::RichText::new(label).heading());

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

    fn draw_fixed_button_and_react(
        &mut self,
        ui: &mut Ui,
        (x, y): (f32, f32),
        tag: &str,
        var: i8,
        texture: &TextureHandle,
        color: Color32,
    ) {
        egui::Area::new(Id::new(tag))
            .fixed_pos(egui::pos2(x, y))
            .show(ui.ctx(), |ui| {
                let parameters = (
                    &self.selected_origin_airport.clone(),
                    &self.selected_destination_airport.clone(),
                    &self.date_selected.clone(),
                    &self.flight_selected.clone(),
                    &self.status_selected.clone(),
                );
                let insert_update_button = insert_or_update_button_can_be_pressed(parameters);
                let delete_button = delete_button_can_be_pressed(parameters);

                if ui.add(egui::Button::new(tag).fill(color)).clicked() {
                    let (query_origin, query_destination) =
                        match_and_get_queries(var, insert_update_button, delete_button, parameters);

                    if !query_origin.is_empty() && !query_destination.is_empty() {
                        let rx: &Receiver<Vec<String>> =
                            self.connection_server.get_channel_to_receive();

                        let _ = self
                            .connection_server
                            .make_query(query_origin, config::QUORUM);
                        let _ = rx.recv().unwrap();
                        let _ = self
                            .connection_server
                            .make_query(query_destination, config::QUORUM);
                        let _ = rx.recv().unwrap();
                        println!("Ok receive response query");

                        if let Some(airport) =
                            self.airports.clone().get(&self.selected_origin_airport)
                        {
                            let (screen_x, screen_y) = get_coordinate_positioning_on_map(
                                airport.coordinates,
                                texture.size(),
                            );
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

        let parameters = (
            &self.selected_origin_airport.clone(),
            &self.selected_destination_airport.clone(),
            &self.date_selected.clone(),
            &self.flight_selected.clone(),
            &self.status_selected.clone(),
        );

        self.draw_other_box(ui, x, y, "origin", AIRPORT); //aeropuerto
        y = offset_y(y);
        self.draw_other_box(ui, x, y, "destination", AIRPORT2); //aeropuerto2
        y = offset_y(y);
        self.draw_other_box(ui, x, y, "date", DATE); //date
        y = offset_y(y);
        self.draw_other_box(ui, x, y, "flight", FLIGHT); //flight
        y = offset_y(y);
        self.draw_status_box(ui, x, y, "status"); //status
        y = offset_y(y);

        let insert_update_button = insert_or_update_button_can_be_pressed(parameters);
        let delete_button = delete_button_can_be_pressed(parameters);

        let insert_update_color = get_color_button(insert_update_button);
        let delete_color = get_color_button(delete_button);

        self.draw_fixed_button_and_react(
            ui,
            (x, y),
            "INSERT",
            INSERT,
            texture,
            insert_update_color,
        );
        x = offset_x(x);
        self.draw_fixed_button_and_react(ui, (x, y), "DELETE", DELETE, texture, delete_color);
        x = offset_x(x);
        self.draw_fixed_button_and_react(
            ui,
            (x, y),
            "UPDATE",
            UPDATE,
            texture,
            insert_update_color,
        );
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
                ui.label(egui::RichText::new("Username:").heading());
                ui.add(egui::TextEdit::singleline(&mut self.username).hint_text("Enter username"));

                ui.label(egui::RichText::new("Password:").heading());
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Enter password"),
                );

                if ui.button(egui::RichText::new("Login").heading()).clicked() {
                    self.get_user();
                }
                ui.label(&self.login_status);
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(egui::RichText::new("Mapa de Aeropuertos").heading());

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
                            self.selected_origin_airport = airport.name.clone();
                            self.selected_destination_airport = "".to_string();
                            self.flight_selected = "".to_string();
                            self.reset_connection();

                            if !self.date_selected.is_empty() {
                                let _ =
                                    self.add_all_connections(airport, texture, screen_x, screen_y);
                            }
                        } // else if !self.date_selected.is_empty() && !self.selected_origin_airport.is_empty() {
                          //     // TODO: Vemos si se agregaron vuelos por el simulador para el refresh de la pantalla.
                          //     // Esto no funciona:
                          //     self.reset_connection();
                          //     let _ = self.add_all_connections(airport, texture, screen_x, screen_y);
                          // }
                    }

                    self.draw_connections_with_flight(texture, ui);

                    self.draw_flight_table(ui, texture);
                });

                self.super_function(ui, texture);
            }
        });
        // Actualizamos la pantalla para que se muestre la nueva posición de los vuelos del simulador.
        ctx.request_repaint();
    }
}

fn offset_y(y: f32) -> f32 {
    y + 30.0
}

fn offset_x(x: f32) -> f32 {
    x + 100.0
}
