use eframe::{egui, epi};

struct MainApp {
    host: String,
    get_path: String,
    post_path: String,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            host: "127.0.0.1:8080".to_string(),
            get_path: "/games".to_string(),
            post_path: "/games".to_string(),
        }
    }
}

impl epi::App for MainApp {
    fn name(&self) -> &str {
        "Test GUI"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Host: ");
                ui.text_edit_singleline(&mut self.host);
            });

            ui.horizontal(|ui| {
                if ui.button("GET   ").clicked() {
                    println!("press GET");
                }
                ui.text_edit_singleline(&mut self.get_path);
            });
            ui.horizontal(|ui| {
                if ui.button("POST").clicked() {
                    println!("press POST");
                }
                ui.text_edit_singleline(&mut self.post_path);
            });
        });
    }
}


#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = MainApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
