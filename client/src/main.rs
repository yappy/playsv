use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Window};

fn basics() -> (Window, Document, HtmlElement) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    (window, document, body)
}

fn app_main() {
    let (_, document, body) = basics();

    let canvas = document.create_element("canvas").unwrap();
    canvas.set_attribute("width", "640").unwrap();
    canvas.set_attribute("height", "480").unwrap();

    body.append_child(canvas.as_ref()).unwrap();

    let canvas: web_sys::HtmlCanvasElement =
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.set_fill_style(&"green".into());
    context.fill_rect(0.0, 0.0, 640.0, 480.0);

    context.begin_path();
}

fn init() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Init OK");
}

fn main() {
    init();
    app_main();
}
