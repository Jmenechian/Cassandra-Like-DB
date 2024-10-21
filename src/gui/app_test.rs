use chrono::{DateTime, Duration, Local, NaiveDate};
use egui_datepicker::DatePicker;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    value: f32,

    #[serde(skip)] // This how you opt-out of serialization of a field
    
    selected_date: NaiveDate, //DateTime<Local>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Aerolineas Rusticas".to_owned(),
            value: 2.0,
            selected_date: NaiveDate::from_ymd(2024, 10, 18), // Fecha inicial
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("aerolineas rusticas");

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });

            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1;
            //     self.selected_date += Duration::days(137);
            // };

            // Aquí está el DatePicker
            ui.add(DatePicker::new("date_id", &mut self.selected_date));
            ui.label(format!("Fecha seleccionada: {}", self.selected_date));

            ui.separator();

            // ui.add(egui::github_link_file!(
            //     "https://github.com/emilk/eframe_template/blob/main/",
            //     "Source code."
            // ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("Ferris", "https://github.com/taller-1-fiuba-rust/24C2-Ferris");
        ui.label(".");
    });
}


/*
______

egui = "0.29"
egui-datepicker = "0.3.0"
eframe = { version = "0.29", default-features = false, features = [
    "accesskit",     # Make egui compatible with screen readers. NOTE: adds a lot of dependencies.
    "default_fonts", # Embed the default egui fonts.
    "glow",          # Use the glow rendering backend. Alternative: "wgpu".
    "persistence",   # Enable restoring app state when restarting the app.
] }
log = "0.4"

# You only need serde if you want app persistence:
serde = { version = "1", features = ["derive"] }

# native:
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = "0.11"

# web:
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = "0.4"
web-sys = "0.3.70" # to access the DOM (to hide the loading text)

[profile.release]
opt-level = 2 # fast and small wasm

# Optimize all dependencies even in debug builds:
[profile.dev.package."*"]
opt-level = 2


[patch.crates-io]
*/