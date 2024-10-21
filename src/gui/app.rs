use eframe::egui;
use egui::{Color32, FontId, RichText, TextureHandle, Ui};
use image::GenericImageView;

use super::painted::{airport::Airport, airport_coordinates::get_airport_coordinates};

pub struct MapApp {
    airports: Vec<Airport>,
    map_texture: Option<egui::TextureHandle>,
}

impl Default for MapApp {
    fn default() -> Self {
        Self {
            airports: get_airport_coordinates(),
            map_texture: None,
        }
    }
}

impl eframe::App for MapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Mapa de Aeropuertos");

            if self.map_texture.is_none() {
                self.map_texture = Some(load_map_texture(ctx));
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
                
                    // Crear un rectángulo para el botón
                    let button_rect = egui::Rect::from_center_size(
                        egui::pos2(screen_x, screen_y), 
                        egui::vec2(radius_click * 2.0, radius_click * 2.0)
                    );
                
                    // Dibuja el círculo
                    ui.painter().circle_filled(egui::pos2(screen_x, screen_y), radius_paint, egui::Color32::RED);
                
                    // Detectar clic en el botón
                    if ui.interact(button_rect, ui.id(), egui::Sense::click()).clicked() {
                        println!("Aeropuerto clicado: {}", airport.name);
                    }
                
                    // Mostrar el nombre del aeropuerto al lado del botón
                    //let name = egui::RichText::new(&airport.name).color(egui::Color32::WHITE);

                    // Posicionar el nombre
                    let name_pos = egui::pos2(screen_x + radius_paint + 5.0, screen_y); // Ajusta la posición según el radio
                    ui.painter().text(name_pos, egui::Align2::LEFT_CENTER, &airport.name, FontId::default(), Color32::WHITE);
                }
                
            }
        });
    }
}

fn load_map_texture(ctx: &egui::Context) -> egui::TextureHandle {
    // Cargar una imagen de mapa desde un archivo
    let map_image = image::open("/Users/martinalarosa/Desktop/Taller1/24C2-Ferris/src/gui/assets/map_2.png").expect("Failed to load map image");
    let (width, height) = map_image.dimensions();
    let image_rgba = map_image.to_rgba8();
    let texture = ctx.load_texture("map", egui::ColorImage::from_rgba_unmultiplied([width.try_into().unwrap(), height.try_into().unwrap()], &image_rgba), Default::default());
    texture
}

fn _draw_airport(ui: &mut Ui, texture: &TextureHandle, airport: &Airport) {
    let (lat, lon) = airport.coordinates;
    let [width, height] = texture.size();
    let screen_x = (lon + 180.0) * (width as f32 / 360.0);
    let screen_y = (90.0 - lat) * (height as f32 / 180.0);

    let radius_click = 15.0; // Radio del botón
    let radius_paint = 5.0;

    // Crear un rectángulo para el botón
    let button_rect = egui::Rect::from_center_size(
        egui::pos2(screen_x, screen_y), 
        egui::vec2(radius_click * 2.0, radius_click * 2.0)
    );

    // Dibuja el círculo
    ui.painter().circle_filled(egui::pos2(screen_x, screen_y), radius_paint, egui::Color32::RED);

    // Detectar clic en el botón
    if ui.interact(button_rect, ui.id(), egui::Sense::click()).clicked() {
        println!("Aeropuerto clicado: {}", airport.name);
    }

    // Mostrar el nombre del aeropuerto al lado del botón
    ui.label(egui::RichText::new(&airport.name).color(egui::Color32::WHITE))
    .on_hover_text("Información del aeropuerto");
    // Posicionar el nombre
    let name_pos = egui::pos2(screen_x + radius_paint + 5.0, screen_y); // Ajusta la posición según el radio
    ui.painter().text(name_pos, egui::Align2::LEFT_CENTER, &airport.name, FontId::default(), Color32::WHITE);
}

fn _draw_airport_2(ui: &mut Ui, texture: &TextureHandle, airport: &Airport) {
    let (lat, lon) = airport.coordinates;
    // let x: f32 = *texture.size().first().unwrap() as f32;
    // let y: f32 = *texture.size().last().unwrap() as f32;
    let [width, height] = texture.size();
    let x = width as f32;
    let y = height as f32;
    let screen_x = (lon + 180.0) * (x / 360.0);
    let screen_y = (90.0 - lat) * (y / 180.0);
    let radius = 5.0;

    // Crear un botón circular
    //let _ = ui.button(egui::RichText::new(&airport.name).size(10.0));

    // Personalizar la apariencia del botón
    ui.painter().circle_filled(
        egui::pos2(screen_x, screen_y), 
        radius, 
        egui::Color32::RED
    );

    let name = RichText::new(&airport.name).heading().color(egui::Color32::WHITE);

    // Posicionar el botón
    let button_rect = egui::Rect::from_min_max(
            egui::pos2(screen_x - 50.0, screen_y - 10.0),
            egui::pos2(screen_x + 100.0, screen_y)
    );
    ui.put(button_rect, egui::Label::new(name));

    let button_rect = egui::Rect::from_center_size(
        egui::pos2(screen_x, screen_y),
        egui::vec2(radius * 2.0, radius * 2.0),
    );

    let response = ui.interact(button_rect, ui.id(), egui::Sense::click());

    if response.clicked() {
        println!("Aeropuerto clicado: {}", airport.name);
    }

    // ---------------------------------------------

        //ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
    // ui.put(egui::Rect::from_min_max(
    //     egui::pos2(screen_x - 50.0, screen_y - 10.0),
    //     egui::pos2(screen_x + 100.0, screen_y)
    // ), egui::Label::new(name));


    // Crear un botón circular
    // if ui.add(egui::Button::new("•").frame(false)).interact(button_rect).clicked() {
    //     println!("Aeropuerto clicado: {}", airport.name);
    // }





//     // Crea un área para detectar clics
//     let button_rect = Rect::from_center_size(Pos2::new(screen_x, screen_y), egui::vec2(radius * 2.0, radius * 2.0));
//     let response: Response = ui.interact(button_rect, Id::new("circular_button"), Sense::click());

//     // Si se hace clic, cambia el estado
//     if response.clicked() {
//         pressed = &!pressed;
//         println!("Botón clicado: {}", pressed);
//     }
}