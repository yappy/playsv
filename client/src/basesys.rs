use std::{cell::RefCell, collections::VecDeque, panic, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, HtmlInputElement,
    KeyboardEvent, MouseEvent, Window,
};

use crate::asset::Assets;

// This should be safe because js event handlers will be
// executed serially on a single thread.
pub static mut PANIC_FLAG: bool = false;

fn is_panic() -> bool {
    unsafe { PANIC_FLAG }
}

fn set_panic() {
    unsafe {
        PANIC_FLAG = true;
    }
}

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
    fn on_ready(&mut self) {}

    fn frame(&mut self) {}
    fn render(&mut self, _context: &CanvasRenderingContext2d, _width: u32, _height: u32) {}

    fn on_key_down(&mut self, _event: &KeyboardEvent) {}
    fn on_key_up(&mut self, _event: &KeyboardEvent) {}
    fn on_mouse_down(&mut self, _event: &MouseEvent, _x: i32, _y: i32) {}
    fn on_mouse_up(&mut self, _event: &MouseEvent, _x: i32, _y: i32) {}
    fn on_mouse_click(&mut self, _event: &MouseEvent, _x: i32, _y: i32) {}

    fn on_debug_command(&mut self, _cmdline: &str) {}
}

pub struct BaseSys<T, F> {
    assets: Option<Assets>,
    app: Option<T>,
    app_factory: F,
    front_canvas: HtmlCanvasElement,
    back_canvas: HtmlCanvasElement,
    debug_cmd: HtmlInputElement,
    canvas_w: u32,
    canvas_h: u32,

    cmd_buffer: VecDeque<String>,
    cmd_index: usize,
}

impl<T, F> BaseSys<T, F>
where
    T: App + 'static,
    F: Fn(Assets) -> T + 'static,
{
    pub fn new(app_factory: F, canvas_w: u32, canvas_h: u32) -> Self {
        // load start
        let assets = Assets::new();

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
        debug_label.set_text_content(Some("Command (F12): "));
        let debug_cmd = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        debug_cmd.set_type("input");
        debug_cmd.set_size(50);
        debug_cmd.set_value("help");

        debug_area.append_child(&debug_label).unwrap();
        debug_area.append_child(&debug_cmd).unwrap();

        body.append_child(&debug_area).unwrap();

        Self {
            assets: Some(assets),
            app: None,
            app_factory,
            front_canvas,
            back_canvas,
            debug_cmd,
            canvas_w,
            canvas_h,
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
        if let Some(ref mut assets) = &mut self.assets {
            if assets.all_images_loaded() {
                let mut app = (self.app_factory)(self.assets.take().unwrap());
                app.on_ready();
                self.app = Some(app);
            }
            return;
        }

        if let Some(ref mut app) = &mut self.app {
            let context = context2d(&self.back_canvas);

            app.frame();

            context.save();
            app.render(&context, self.canvas_w, self.canvas_h);
            context.restore();
        }
    }

    fn on_animation_frame(&mut self) {
        self.flip();
    }

    fn on_keydown(&mut self, event: &KeyboardEvent) {
        // ignore keeping pressed
        if event.repeat() {
            return;
        }

        if let Some(ref mut app) = &mut self.app {
            log::info!("Key down: {}", event.code());
            app.on_key_down(event);
        }
    }

    fn on_keyup(&mut self, event: &KeyboardEvent) {
        if let Some(ref mut app) = &mut self.app {
            log::info!("Key up: {}", event.code());
            app.on_key_up(event);
        }
    }

    fn translate_mouse_pos(elem: &HtmlCanvasElement, event: &MouseEvent) -> (i32, i32) {
        let rect = elem.get_bounding_client_rect();
        let x = event.client_x();
        let y = event.client_y();

        (x - rect.x() as i32, y - rect.y() as i32)
    }

    fn on_mousedown(&mut self, event: &MouseEvent) {
        if let Some(ref mut app) = &mut self.app {
            let (x, y) = Self::translate_mouse_pos(&self.front_canvas, event);
            log::info!("Mouse down: {} ({}, {})", event.button(), x, y);
            app.on_mouse_down(event, x, y);
        }
    }

    fn on_mouseup(&mut self, event: &MouseEvent) {
        if let Some(ref mut app) = &mut self.app {
            let (x, y) = Self::translate_mouse_pos(&self.front_canvas, event);
            log::info!("Mouse up: {} ({}, {})", event.button(), x, y);
            app.on_mouse_up(event, x, y);
        }
    }

    fn on_click(&mut self, event: &MouseEvent) {
        if let Some(ref mut app) = &mut self.app {
            let (x, y) = Self::translate_mouse_pos(&self.front_canvas, event);
            log::info!("Mouse click: {} ({}, {})", event.button(), x, y);
            app.on_mouse_click(event, x, y);
        }
    }

    fn on_debug_keydown(&mut self, event: &KeyboardEvent) {
        if let Some(ref mut app) = &mut self.app {
            let key = event.key();
            let text = self.debug_cmd.value();

            match key.as_str() {
                "Enter" => {
                    self.cmd_buffer.remove(self.cmd_index);
                    self.cmd_buffer.push_front(text.clone());
                    self.cmd_buffer.push_front("".to_string());
                    self.cmd_index = 0;
                    self.debug_cmd.set_value("");

                    app.on_debug_command(&text);
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
    }

    pub fn start(self) {
        let basesys = Rc::new(RefCell::new(self));

        let (window, _document, _body) = basics();

        // window.setInterval()
        let cb: Closure<dyn FnMut()> = {
            let basesys = basesys.clone();
            Closure::new(move || {
                if !is_panic() {
                    basesys.borrow_mut().on_interval();
                }
            })
        };
        let _id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1000 / 60,
            )
            .unwrap();
        cb.forget();

        // window.requestAnimationFrame()
        type Cb = Closure<dyn FnMut()>;
        let pcb: Rc<RefCell<Option<Cb>>> = Rc::new(RefCell::new(None));
        let pcb_move = pcb.clone();
        let basesys_move = basesys.clone();
        *pcb.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
            if !is_panic() {
                basesys_move.borrow_mut().on_animation_frame();
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(
                        pcb_move.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        }));
        let _id = window
            .request_animation_frame(pcb.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();

        // front_canvas.addEventListener("keydown")
        let cb = {
            let basesys = basesys.clone();
            Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                if !is_panic() {
                    basesys.borrow_mut().on_keydown(&event);
                }
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
                if !is_panic() {
                    basesys.borrow_mut().on_keyup(&event);
                }
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
                if !is_panic() {
                    basesys.borrow_mut().on_mousedown(&event);
                }
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
                if !is_panic() {
                    basesys.borrow_mut().on_mouseup(&event);
                }
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
                if !is_panic() {
                    basesys.borrow_mut().on_click(&event);
                }
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
                if !is_panic() {
                    basesys.borrow_mut().on_debug_keydown(&event);
                }
            })
        };
        basesys
            .borrow()
            .debug_cmd
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();

        // Install a new panic handler
        let old = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            old(info);
            set_panic();
            let _ = web_sys::window()
                .unwrap()
                .alert_with_message("Fatal Error: See the debug console.");
        }));
    }
}
