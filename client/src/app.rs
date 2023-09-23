use std::sync::Mutex;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, HtmlElement, Window,
};

const CANVAS_ID: &str = "main_canvas";
const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

static APP: Mutex<Option<App>> = Mutex::new(None);

fn basics() -> (Window, Document, HtmlElement) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    (window, document, body)
}

#[derive(Debug, Default)]
struct App {
    canvas_id: String,
    canvas_w: u32,
    canvas_h: u32,
    interval_id: Option<i32>,
    frame: u64,
}

impl App {
    fn new(canvas_id: &str, canvas_w: u32, canvas_h: u32) -> Self {
        let (_window, document, body) = basics();

        let canvas = document.create_element("canvas").unwrap();
        canvas.set_attribute("id", &canvas_id.to_string()).unwrap();
        canvas
            .set_attribute("width", &canvas_w.to_string())
            .unwrap();
        canvas
            .set_attribute("height", &canvas_h.to_string())
            .unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        body.append_child(canvas.as_ref()).unwrap();

        App {
            canvas_id: canvas_id.to_string(),
            canvas_w,
            canvas_h,
            interval_id: None,
            frame: 0,
        }
    }

    fn context2d(&self) -> CanvasRenderingContext2d {
        let (_window, document, _body) = basics();

        let canvas = document
            .get_element_by_id(&self.canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        context
    }

    fn interval_handler() {
        let mut app = APP.lock().unwrap();
        let app = app.as_mut().unwrap();

        let context = app.context2d();

        let t = app.frame as u8;
        let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        context.set_fill_style(&color.into());
        context.fill_rect(0.0, 0.0, app.canvas_w as f64, app.canvas_h as f64);

        app.frame += 1;
    }

    fn start_interval(&mut self) {
        assert!(self.interval_id.is_none());

        let (window, _, _) = basics();

        let cb: Closure<dyn FnMut()> = Closure::new(move || Self::interval_handler());
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1000 / 60,
            )
            .unwrap();
        cb.forget();
        log::info!("setInterval: {id}");

        self.interval_id = Some(id);
    }
}

pub fn app_main() {
    let app = App::new(CANVAS_ID, CANVAS_W, CANVAS_H);
    {
        let mut lock = APP.lock().unwrap();
        *lock = Some(app);
        let app = lock.as_mut().unwrap();
        app.start_interval();
    }
}
