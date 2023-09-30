mod asset;
mod basesys;
mod net;
mod mainapp;
mod jsif;

fn init() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Panic Hook and Init OK");
}

fn main() {
    init();
    mainapp::app_main();
}
