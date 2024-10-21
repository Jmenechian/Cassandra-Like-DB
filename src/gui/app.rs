use walkers::{sources::OpenStreetMap, HttpTiles, Map, MapMemory, Position};
use egui::{CentralPanel, Color32, Context, Pos2};
use eframe::{App, Frame};

const TILE_SIZE: u32 = 256;
use std::f64::consts::PI;
pub struct MapApp {
    tiles: HttpTiles,
    pub map_memory: MapMemory,
    airports: Vec<Position>,
}

impl MapApp {
   pub fn new(egui_ctx: Context) -> Self {
        let mut airports = vec![];
        airports.push(Position::from_lon_lat(-58.5, -34.8));
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

    fn mercator_normalized(position: Vec<f64>) -> (f64, f64) { //---> funcion ya definida en la map.rs
        // Project into Mercator (cylindrical map projection).
        let x = position[0].to_radians();
        let y = position[1].to_radians().tan().asinh();
    
        // Scale both x and y to 0-1 range.
        let x = (1. + (x / PI)) / 2.;
        let y = (1. - (y / PI)) / 2.;
    
        (x, y)
    }

    /// Project `position` into pixels on the viewport. ---> funcion ya definida en la map.rs
    pub fn project(position: Vec<f64>, zoom: f64) -> Pos2 {
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
            ).zoom_gesture(true); //.with_plugin(plugins::CustomShapes {});
            ui.add(map);
            let point: Vec<f64> = vec![-76.97, 128.57];
            let projected_position = MapApp::project(point, self.map_memory.zoom().into());
            let painter = ui.painter();
            let radius = 20.;
            painter.circle_filled(
                projected_position,
                radius,
                Color32::RED //.gamma_multiply(if hovered { 0.5 } else { 0.2 }),
            );
        }
    );
    }
}
