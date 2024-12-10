use aerolineas_rusticas::gui::app::MapApp;
use std::env;
use std::process::exit;

/// Binario que permite la ejecución de la Interfaz gráfica de usuario.
fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Last digit parameter of the port is missing.");
        exit(1);
    }
    let port = args[1].to_string();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([2000.0, 1000.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../gui/assets/plane.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Aiports Map",
        options,
        Box::new(|_cc| Ok(Box::new(MapApp::new(port)))),
    )
}
