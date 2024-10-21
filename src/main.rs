use aerolineas_rusticas::gui::app::MapApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([2000.0, 1000.0])
            .with_min_inner_size([1000.0, 500.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("gui/assets/avion.png")[..])
                .expect("Failed to load icon"),
            ),
        ..Default::default() 
    };

    eframe::run_native(
        "Mapa de Aeropuertos",
        options,
        Box::new(|_cc| Ok(Box::new(MapApp::default()))),
    )
}
