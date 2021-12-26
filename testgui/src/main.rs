use eframe::{egui, epi};
use std::thread;
use std::sync::mpsc;

enum Method {
    GET, POST,
}

struct NetworkTask {
    url: String,
    method: Method,
    body: String,
}

struct MainApp {
    tx: mpsc::SyncSender<NetworkTask>,
    rx: mpsc::Receiver<String>,
    host: String,
    get_path: String,
    post_path: String,
    res_msg: String,
}

impl MainApp {
    fn new(tx: mpsc::SyncSender<NetworkTask>, rx: mpsc::Receiver<String>) -> Self {
        Self {
            tx, rx,
            host: "http://127.0.0.1:8080".to_string(),
            get_path: "/games".to_string(),
            post_path: "/games".to_string(),
            res_msg: "".to_string(),
        }
    }
}

impl epi::App for MainApp {
    fn name(&self) -> &str {
        "Test GUI"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        // network thread communication
        while let Ok(msg) = self.rx.try_recv() {
            self.res_msg = msg;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Host: ");
                ui.text_edit_singleline(&mut self.host);
            });

            ui.horizontal(|ui| {
                if ui.button("GET   ").clicked() {
                    let url = self.host.clone() + &self.get_path;
                    let task = NetworkTask {url, method: Method::GET, body: "".to_string()};
                    let _ = self.tx.try_send(task);
                    self.res_msg = "Please wait...".to_string();
                }
                ui.text_edit_singleline(&mut self.get_path);
            });
            ui.horizontal(|ui| {
                if ui.button("POST").clicked() {
                    let url = self.host.clone() + &self.post_path;
                    let task = NetworkTask {url, method: Method::POST, body: "".to_string()};
                    let _ = self.tx.try_send(task);
                    self.res_msg = "Please wait...".to_string();
                }
                ui.text_edit_singleline(&mut self.post_path);
            });

            ui.code(&self.res_msg);
        });

        ctx.request_repaint();
    }
}

fn network_thread_entry(tx: mpsc::Sender<String>, rx: mpsc::Receiver<NetworkTask>) {
    let client = reqwest::blocking::Client::new();

    for task in rx {
        println!("{}", task.url);
        let resp = match task.method {
            Method::GET =>
                client.get(task.url)
                    .send(),
            Method::POST =>
                client.post(task.url)
                    .body(task.body)
                    .send(),
        };
        if let Ok(resp) = resp {
            let status = resp.status();
            let body = resp.text();
            if let Ok(body) = body {
                tx.send(format!("Status: {:?}\n\n{}", status, body)).unwrap();
            }
            else if let Err(_e) = body {
                tx.send("Body Text Error".to_string()).unwrap();
            }
        }
        else if let Err(_e) = resp {
            tx.send("Network Error".to_string()).unwrap();
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // App to Network
    let (a2n_tx, a2n_rx) = mpsc::sync_channel(1);
    // Network to App
    let (n2a_tx, n2a_rx) = mpsc::channel();
    {
        thread::spawn(move || {
            network_thread_entry(n2a_tx, a2n_rx);
        });
    }
    {
        let app = MainApp::new(a2n_tx, n2a_rx);
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(Box::new(app), native_options);
    }
}
