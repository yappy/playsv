use web_sys::window;

fn app_main() {
    let document = window().and_then(|win| win.document()).expect("Could not access document");
    let body = document.body().expect("Could not access document.body");
    let text_node = document.create_text_node("Hello, world from Vanilla Rust!");
    body.append_child(text_node.as_ref()).expect("Failed to append text");
}

fn init() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

fn main() {
    init();
    app_main();
}
