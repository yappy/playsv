mod asset;
mod basesys;
mod mainapp;
mod net;
mod testmode;

fn init() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Panic Hook and Init OK");
}

fn main() {
    init();
    mainapp::app_main();
}
