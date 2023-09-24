use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, Window};

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

fn basics() -> (Window, Document, HtmlElement) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    (window, document, body)
}

#[derive(Debug)]
struct App {
    canvas: HtmlCanvasElement,
    canvas_w: u32,
    canvas_h: u32,
    interval_id: Option<i32>,
    frame: u64,
}

impl App {
    fn new(canvas_w: u32, canvas_h: u32) -> Self {
        let (_window, document, body) = basics();

        let canvas = document.create_element("canvas").unwrap();
        canvas
            .set_attribute("width", &canvas_w.to_string())
            .unwrap();
        canvas
            .set_attribute("height", &canvas_h.to_string())
            .unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        body.append_child(canvas.as_ref()).unwrap();

        App {
            canvas,
            canvas_w,
            canvas_h,
            interval_id: None,
            frame: 0,
        }
    }

    fn context2d(&self) -> CanvasRenderingContext2d {
        self.canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap()
    }

    fn interval_handler(&mut self) {
        let context = self.context2d();

        let t = self.frame as u8;
        let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        context.set_fill_style(&color.into());
        context.fill_rect(0.0, 0.0, self.canvas_w as f64, self.canvas_h as f64);

        self.frame += 1;
    }

    fn start(self) {
        assert!(self.interval_id.is_none());

        let app = Rc::new(RefCell::new(self));

        let (window, _, _) = basics();

        let cb: Closure<dyn FnMut()> = {
            let app = app.clone();
            Closure::new(move || {
                app.borrow_mut().interval_handler();
            })
        };
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1000 / 60,
            )
            .unwrap();
        cb.forget();
        log::info!("setInterval: {id}");

        app.borrow_mut().interval_id = Some(id);
    }
}

pub fn app_main() {
    let app = App::new(CANVAS_W, CANVAS_H);
    app.start();
}
