use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, HtmlInputElement,
    KeyboardEvent, MouseEvent, Window,
};

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

pub trait App {
    fn frame(&mut self) {}
    fn render(&mut self, _context: &CanvasRenderingContext2d, _width: u32, _height: u32) {}

    fn init(&mut self) {}
    fn on_key_down(&mut self, _event: &KeyboardEvent) {}
    fn on_key_up(&mut self, _event: &KeyboardEvent) {}
    fn on_mouse_down(&mut self, _event: &MouseEvent) {}
    fn on_mouse_up(&mut self, _event: &MouseEvent) {}
    fn on_mouse_click(&mut self, _event: &MouseEvent) {}

    fn on_debug_command(&mut self, _cmdline: &str) {}
}

#[derive(Debug)]
pub struct BaseSys<T> {
    app: T,
    front_canvas: HtmlCanvasElement,
    back_canvas: HtmlCanvasElement,
    debug_cmd: HtmlInputElement,
    canvas_w: u32,
    canvas_h: u32,
    interval_id: Option<i32>,

    cmd_buffer: VecDeque<String>,
    cmd_index: usize,
}

impl<T: App + 'static> BaseSys<T> {
    pub fn new(app: T, canvas_w: u32, canvas_h: u32) -> Self {
        let (_window, document, body) = basics();

        let create_canvas = || {
            let canvas = document
                .create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();
            canvas.set_width(canvas_w);
            canvas.set_height(canvas_h);

            canvas
        };

        let front_canvas = create_canvas();
        let back_canvas = create_canvas();

        body.append_child(&front_canvas).unwrap();

        let debug_area = document.create_element("div").unwrap();
        let debug_label = document.create_element("label").unwrap();
        debug_label.set_text_content(Some("Command: "));
        let debug_cmd = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        debug_cmd.set_type("input");
        debug_cmd.set_size(50);

        debug_area.append_child(&debug_label).unwrap();
        debug_area.append_child(&debug_cmd).unwrap();

        body.append_child(&debug_area).unwrap();

        Self {
            app,
            front_canvas,
            back_canvas,
            debug_cmd,
            canvas_w,
            canvas_h,
            interval_id: None,
            cmd_buffer: VecDeque::from(["".to_string()]),
            cmd_index: 0,
        }
    }

    fn flip(&self) {
        // Copy back => front
        let context = context2d(&self.front_canvas);
        context
            .draw_image_with_html_canvas_element(&self.back_canvas, 0.0, 0.0)
            .unwrap();
    }

    fn on_interval(&mut self) {
        let context = context2d(&self.back_canvas);

        self.app.frame();
        self.app.render(&context, self.canvas_w, self.canvas_h);

        // TODO animationFrame would be better
        self.flip();
    }

    fn on_keydown(&mut self, event: &KeyboardEvent) {
        if event.repeat() {
            return;
        }
        log::info!("Key down: {}", event.code());
        self.app.on_key_down(event);
    }

    fn on_keyup(&mut self, event: &KeyboardEvent) {
        log::info!("Key up: {}", event.code());
        self.app.on_key_up(event);
    }

    fn on_mousedown(&mut self, event: &MouseEvent) {
        log::info!(
            "Mouse down: {} ({}, {})",
            event.button(),
            event.client_x(),
            event.client_y()
        );
        self.app.on_mouse_down(event);
    }

    fn on_mouseup(&mut self, event: &MouseEvent) {
        log::info!(
            "Mouse up: {} ({}, {})",
            event.button(),
            event.client_x(),
            event.client_y()
        );
        self.app.on_mouse_up(event);
    }

    fn on_click(&mut self, event: &MouseEvent) {
        log::info!(
            "Mouse click: {} ({}, {})",
            event.button(),
            event.client_x(),
            event.client_y()
        );
        self.app.on_mouse_click(event);
    }

    fn on_debug_keydown(&mut self, event: &KeyboardEvent) {
        let key = event.key();
        let text = self.debug_cmd.value();

        match key.as_str() {
            "Enter" => {
                self.cmd_buffer.remove(self.cmd_index);
                self.cmd_buffer.push_front(text.clone());
                self.cmd_buffer.push_front("".to_string());
                self.cmd_index = 0;
                self.debug_cmd.set_value("");

                self.app.on_debug_command(&text);
            }
            "Down" | "ArrowDown" => {
                self.cmd_buffer[self.cmd_index] = text;
                let new_index = self.cmd_index.saturating_sub(1);
                let new_index = new_index.clamp(0, self.cmd_buffer.len() - 1);
                self.cmd_index = new_index;
                let new_text = self.cmd_buffer[new_index].as_str();
                self.debug_cmd.set_value(new_text);
            }
            "Up" | "ArrowUp" => {
                self.cmd_buffer[self.cmd_index] = text;
                let new_index = self.cmd_index.saturating_add(1);
                let new_index = new_index.clamp(0, self.cmd_buffer.len() - 1);
                self.cmd_index = new_index;
                let new_text = self.cmd_buffer[new_index].as_str();
                self.debug_cmd.set_value(new_text);
            }
            _ => {}
        }
    }

    pub fn start(self) {
        assert!(self.interval_id.is_none());

        let basesys = Rc::new(RefCell::new(self));

        let (window, _document, _body) = basics();

        // window.setInterval()
        let cb: Closure<dyn FnMut()> = {
            let basesys = basesys.clone();
            Closure::new(move || {
                basesys.borrow_mut().on_interval();
            })
        };
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1000 / 60,
            )
            .unwrap();
        cb.forget();
        basesys.borrow_mut().interval_id = Some(id);
        log::info!("setInterval: {id}");

        // front_canvas.addEventListener("keydown")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                basesys.borrow_mut().on_keydown(&event);
            })
        };
        basesys
            .borrow()
            .front_canvas
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // front_canvas.addEventListener("keyup")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                basesys.borrow_mut().on_keyup(&event);
            })
        };
        basesys
            .borrow()
            .front_canvas
            .add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // front_canvas.addEventListener("mouseup")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                basesys.borrow_mut().on_mousedown(&event);
            })
        };
        basesys
            .borrow()
            .front_canvas
            .add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // front_canvas.addEventListener("mousedown")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                basesys.borrow_mut().on_mouseup(&event);
            })
        };
        basesys
            .borrow()
            .front_canvas
            .add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // front_canvas.addEventListener("click")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                basesys.borrow_mut().on_click(&event);
            })
        };
        basesys
            .borrow()
            .front_canvas
            .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // debug_cmd.addEventListener("keydown")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                basesys.borrow_mut().on_debug_keydown(&event);
            })
        };
        basesys
            .borrow()
            .debug_cmd
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        basesys.borrow_mut().app.init();
    }
}
