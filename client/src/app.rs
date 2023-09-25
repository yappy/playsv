use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, KeyboardEvent, Window,
};

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

fn basics() -> (Window, Document, HtmlElement) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    (window, document, body)
}

fn context2d(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap()
}

#[derive(Debug)]
struct App {
    front_canvas: HtmlCanvasElement,
    back_canvas: HtmlCanvasElement,
    canvas_w: u32,
    canvas_h: u32,
    interval_id: Option<i32>,
    frame: u64,
}

impl App {
    fn new(canvas_w: u32, canvas_h: u32) -> Self {
        let (_window, document, body) = basics();

        let create_canvas = || {
            let canvas = document.create_element("canvas").unwrap();
            canvas
                .set_attribute("width", &canvas_w.to_string())
                .unwrap();
            canvas
                .set_attribute("height", &canvas_h.to_string())
                .unwrap();

            canvas.dyn_into::<HtmlCanvasElement>().unwrap()
        };

        let front_canvas = create_canvas();
        let back_canvas = create_canvas();

        body.append_child(&front_canvas).unwrap();

        App {
            front_canvas,
            back_canvas,
            canvas_w,
            canvas_h,
            interval_id: None,
            frame: 0,
        }
    }

    fn flip(&self) {
        let context = context2d(&self.front_canvas);
        context
            .draw_image_with_html_canvas_element(&self.back_canvas, 0.0, 0.0)
            .unwrap();
    }

    fn on_interval(&mut self) {
        let context = context2d(&self.back_canvas);

        let t = self.frame as u8;
        let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        context.set_fill_style(&color.into());
        context.fill_rect(0.0, 0.0, self.canvas_w as f64, self.canvas_h as f64);

        self.frame += 1;

        self.flip();
    }

    fn on_keydown(&mut self, event: &KeyboardEvent) {
        log::info!("Key down: {}", event.code());
    }

    fn on_keyup(&mut self, event: &KeyboardEvent) {
        log::info!("Key up: {}", event.code());
    }

    fn start(self) {
        assert!(self.interval_id.is_none());

        let app = Rc::new(RefCell::new(self));

        let (window, document, _) = basics();

        // window.setInterval()
        let cb: Closure<dyn FnMut()> = {
            let app = app.clone();
            Closure::new(move || {
                app.borrow_mut().on_interval();
            })
        };
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1000 / 60,
            )
            .unwrap();
        cb.forget();
        app.borrow_mut().interval_id = Some(id);
        log::info!("setInterval: {id}");

        // document.addEventListener("keydown")
        let cb = {
            let app = app.clone();

            Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                if event.repeat() {
                    return;
                }
                app.borrow_mut().on_keydown(&event);
            })
        };
        document
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // document.addEventListener("keyup")
        let cb = {
            let app = app.clone();

            Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                app.borrow_mut().on_keyup(&event);
            })
        };
        document
            .add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }
}

pub fn app_main() {
    let app = App::new(CANVAS_W, CANVAS_H);
    app.start();
}
