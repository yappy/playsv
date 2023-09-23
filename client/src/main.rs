mod app;

fn init() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Init OK");
}

fn main() {
    init();
    app::app_main();
}
