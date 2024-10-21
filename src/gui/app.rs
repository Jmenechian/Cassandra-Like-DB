use walkers::{sources::OpenStreetMap, HttpTiles, Map, MapMemory, Plugin, Position, Projector, Tiles};
use egui::{CentralPanel, Color32, Context, Pos2, Vec2};
use eframe::{App, Frame};
use std::collections::HashMap;

use super::{painted::airport, plugins::{self, ImagesPluginData, CustomShapes}, windows::{Provider, providers}};
pub(crate) const TILE_SIZE: u32 = 256;
use std::f64::consts::PI;
pub struct MapApp {
    tiles: HttpTiles,
    pub map_memory: MapMemory,
    airports: Vec<Position>,
}

impl MapApp {
   pub fn new(egui_ctx: Context) -> Self {
        let mut airports = vec![];
        airports.push(Position::from_lon_lat(34.8, 58.5));
        airports.push(Position::from_lon_lat(-68.8, -32.8));

        Self {
            tiles: HttpTiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
            airports: airports,
        }
    }

    // Método para agregar un punto
    pub fn add_point(&mut self, lon: f64, lat: f64) {
        self.airports.push(Position::from_lon_lat(lon, lat));
    }

    fn mercator_normalized(position: Position) -> (f64, f64) { //---> funcion ya definida en la map.rs
        // Project into Mercator (cylindrical map projection).
        let x = position.lon().to_radians();
        let y = position.lat().to_radians().tan().asinh();
    
        // Scale both x and y to 0-1 range.
        let x = (1. + (x / PI)) / 2.;
        let y = (1. - (y / PI)) / 2.;
    
        (x, y)
    }

    /// Project `position` into pixels on the viewport. ---> funcion ya definida en la map.rs
    pub fn project(position: Position, zoom: f64) -> Pos2 {
        let (x, y) = MapApp::mercator_normalized(position);

        // Map that into a big bitmap made out of web tiles.
        let number_of_pixels = 2f64.powf(zoom) * (TILE_SIZE as f64);
        let x = x * number_of_pixels;
        let y = y * number_of_pixels;
        
        egui::pos2(x as f32, y as f32)
    }
}

impl App for MapApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let my_position = Position::from_lon_lat(-58.536304, -34.8157064); //CABA
            let map = Map::new(
                Some(&mut self.tiles),
                &mut self.map_memory,
                my_position,
            ).zoom_gesture(true).with_plugin(plugins::CustomShapes {});
            ui.add(map);

            // Dibuja los puntos en el mapa
            //for point in &self.airports {
                // Tomo de ejemplo: https://github.com/podusowski/walkers/tree/main/demo y de ahí plugins::CustomShapes.
                let point = (97.57, -76.97);

                let position = Position::from_lon_lat(point.0, point.1);
                let projected_position = MapApp::project(position, self.map_memory.zoom().into());
                let painter = ui.painter();
                let radius = 20.;
                painter.circle_filled(
                    projected_position,
                    radius,
                    Color32::RED //.gamma_multiply(if hovered { 0.5 } else { 0.2 }),
                );

                //let projector = Projector::new(b, &self.map_memory, position);
                // let vec_2 = projector.project(position);

                // let position = Position::from_lon_lat(vec_2.x as f64, vec_2.y as f64);
                // let projector = Projector::new(button_rect, &self.map_memory, position);
                // let button_rect = egui::Rect::from_center_size(
                //     egui::pos2(vec_2.x as f32, vec_2.y as f32), 
                //     egui::vec2(radius_click * 2.0, radius_click * 2.0)
                // );


                // let button_rect = egui::Rect::from_center_size(
                //     egui::pos2(position.lon() as f32, position.lat() as f32), 
                //     egui::vec2(radius_click * 2.0, radius_click * 2.0)
                // );

                // let response = ui.button("Click me!");
                //let painter = ui.painter().with_clip_rect(b);

                // Project it into the position on the screen.
                // let position = projector.project(position).to_pos2();
                //
                //let position = projector.unproject(Vec2::new(point.lon() as f32, point.lat() as f32));
                //let position = egui::pos2(position.lon() as f32, position.lat() as f32);
                //let position = egui::pos2(point.0 as f32, point.1 as f32);
        
                // let hovered = response
                //     .hover_pos()
                //     .map(|hover_pos| hover_pos.distance(position) < radius)
                //     .unwrap_or(false);
        
                

                // let mut custom_shapes = CustomShapes {};
                // custom_shapes.run(&response, painter, &projector);
            //}

            
            // Detectar click en el mapa
            if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
                if let Some(pos) = ctx.pointer_interact_pos() {
                    let b = egui::Rect::NOTHING;
                    let position = Position::from_lon_lat(pos.x as f64, pos.y as f64);
                    let projector = Projector::new(b, &self.map_memory, position);
                    let position = projector.unproject(Vec2::new(pos.x, pos.y));
                    println!("Se marcó en el mapa - post: {:?}, position: {:?}", pos, position);
                }
            }
        }
    );
    }
}




// use eframe::egui;
// use egui::{Color32, FontId, RichText, TextureHandle, Ui};
// use image::GenericImageView;

// use super::painted::{airport::Airport, airport_coordinates::get_airport_coordinates};

// pub struct MapApp {
//     airports: Vec<Airport>,
//     map_texture: Option<egui::TextureHandle>,
// }

// impl Default for MapApp {
//     fn default() -> Self {
//         Self {
//             airports: get_airport_coordinates(),
//             map_texture: None,
//         }
//     }
// }

// impl eframe::App for MapApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.label("Mapa de Aeropuertos");

//             if self.map_texture.is_none() {
//                 self.map_texture = Some(load_map_texture(ctx));
//             }

//             if let Some(texture) = &self.map_texture {
//                 // Mostrar el mapa
//                 ui.image(texture);

//                 // Dibujar los aeropuertos
//                 for airport in &self.airports {
//                     //draw_airport(ui, texture, airport);
//                     let (lat, lon) = airport.coordinates;
//                     let [width, height] = texture.size();
//                     let screen_x = (lon + 180.0) * (width as f32 / 360.0);
//                     let screen_y = (90.0 - lat) * (height as f32 / 180.0);
                
//                     let radius_click = 15.0; // Radio del botón
//                     let radius_paint = 5.0;
                
//                     // Crear un rectángulo para el botón
//                     let button_rect = egui::Rect::from_center_size(
//                         egui::pos2(screen_x, screen_y), 
//                         egui::vec2(radius_click * 2.0, radius_click * 2.0)
//                     );
                
//                     // Dibuja el círculo
//                     ui.painter().circle_filled(egui::pos2(screen_x, screen_y), radius_paint, egui::Color32::RED);
                
//                     // Detectar clic en el botón
//                     if ui.interact(button_rect, ui.id(), egui::Sense::click()).clicked() {
//                         println!("Aeropuerto clicado: {}", airport.name);
//                     }
                
//                     // Mostrar el nombre del aeropuerto al lado del botón
//                     //let name = egui::RichText::new(&airport.name).color(egui::Color32::WHITE);

//                     // Posicionar el nombre
//                     let name_pos = egui::pos2(screen_x + radius_paint + 5.0, screen_y); // Ajusta la posición según el radio
//                     ui.painter().text(name_pos, egui::Align2::LEFT_CENTER, &airport.name, FontId::default(), Color32::WHITE);
//                 }
                
//             }
//         });
//     }
// }

// fn load_map_texture(ctx: &egui::Context) -> egui::TextureHandle {
//     // Cargar una imagen de mapa desde un archivo
//     let map_image = image::open("/Users/martinalarosa/Desktop/Taller1/24C2-Ferris/src/gui/assets/map_2.png").expect("Failed to load map image");
//     let (width, height) = map_image.dimensions();
//     let image_rgba = map_image.to_rgba8();
//     let texture = ctx.load_texture("map", egui::ColorImage::from_rgba_unmultiplied([width.try_into().unwrap(), height.try_into().unwrap()], &image_rgba), Default::default());
//     texture
// }

// fn draw_airport(ui: &mut Ui, texture: &TextureHandle, airport: &Airport) {
//     let (lat, lon) = airport.coordinates;
//     let [width, height] = texture.size();
//     let screen_x = (lon + 180.0) * (width as f32 / 360.0);
//     let screen_y = (90.0 - lat) * (height as f32 / 180.0);

//     let radius_click = 15.0; // Radio del botón
//     let radius_paint = 5.0;

//     // Crear un rectángulo para el botón
//     let button_rect = egui::Rect::from_center_size(
//         egui::pos2(screen_x, screen_y), 
//         egui::vec2(radius_click * 2.0, radius_click * 2.0)
//     );

//     // Dibuja el círculo
//     ui.painter().circle_filled(egui::pos2(screen_x, screen_y), radius_paint, egui::Color32::RED);

//     // Detectar clic en el botón
//     if ui.interact(button_rect, ui.id(), egui::Sense::click()).clicked() {
//         println!("Aeropuerto clicado: {}", airport.name);
//     }

//     // Mostrar el nombre del aeropuerto al lado del botón
//     ui.label(egui::RichText::new(&airport.name).color(egui::Color32::WHITE))
//     .on_hover_text("Información del aeropuerto");
//     // Posicionar el nombre
//     let name_pos = egui::pos2(screen_x + radius_paint + 5.0, screen_y); // Ajusta la posición según el radio
//     ui.painter().text(name_pos, egui::Align2::LEFT_CENTER, &airport.name, FontId::default(), Color32::WHITE);
// }

// fn draw_airport_2(ui: &mut Ui, texture: &TextureHandle, airport: &Airport) {
//     let (lat, lon) = airport.coordinates;
//     // let x: f32 = *texture.size().first().unwrap() as f32;
//     // let y: f32 = *texture.size().last().unwrap() as f32;
//     let [width, height] = texture.size();
//     let x = width as f32;
//     let y = height as f32;
//     let screen_x = (lon + 180.0) * (x / 360.0);
//     let screen_y = (90.0 - lat) * (y / 180.0);
//     let radius = 5.0;

//     // Crear un botón circular
//     //let _ = ui.button(egui::RichText::new(&airport.name).size(10.0));

//     // Personalizar la apariencia del botón
//     ui.painter().circle_filled(
//         egui::pos2(screen_x, screen_y), 
//         radius, 
//         egui::Color32::RED
//     );

//     let name = RichText::new(&airport.name).heading().color(egui::Color32::WHITE);

//     // Posicionar el botón
//     let button_rect = egui::Rect::from_min_max(
//             egui::pos2(screen_x - 50.0, screen_y - 10.0),
//             egui::pos2(screen_x + 100.0, screen_y)
//     );
//     ui.put(button_rect, egui::Label::new(name));

//     let button_rect = egui::Rect::from_center_size(
//         egui::pos2(screen_x, screen_y),
//         egui::vec2(radius * 2.0, radius * 2.0),
//     );

//     let response = ui.interact(button_rect, ui.id(), egui::Sense::click());

//     if response.clicked() {
//         println!("Aeropuerto clicado: {}", airport.name);
//     }

//     // ---------------------------------------------

//         //ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
//     // ui.put(egui::Rect::from_min_max(
//     //     egui::pos2(screen_x - 50.0, screen_y - 10.0),
//     //     egui::pos2(screen_x + 100.0, screen_y)
//     // ), egui::Label::new(name));


//     // Crear un botón circular
//     // if ui.add(egui::Button::new("•").frame(false)).interact(button_rect).clicked() {
//     //     println!("Aeropuerto clicado: {}", airport.name);
//     // }





// //     // Crea un área para detectar clics
// //     let button_rect = Rect::from_center_size(Pos2::new(screen_x, screen_y), egui::vec2(radius * 2.0, radius * 2.0));
// //     let response: Response = ui.interact(button_rect, Id::new("circular_button"), Sense::click());

// //     // Si se hace clic, cambia el estado
// //     if response.clicked() {
// //         pressed = &!pressed;
// //         println!("Botón clicado: {}", pressed);
// //     }
// }