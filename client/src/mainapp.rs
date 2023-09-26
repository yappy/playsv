use std::collections::VecDeque;

use crate::basesys::{App, BaseSys};
use web_sys::CanvasRenderingContext2d;

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

struct MainApp {
    frame: u64,

    cmd_buffer: VecDeque<String>,
    cmd_index: usize,
}

impl MainApp {
    fn new() -> Self {
        Self {
            frame: 0,
            cmd_buffer: VecDeque::from(["".to_string()]),
            cmd_index: 0,
        }
    }
}

impl App for MainApp {
    fn frame(&mut self) {
        self.frame += 1;
    }

    fn render(&mut self, context: &CanvasRenderingContext2d, width: u32, height: u32) {
        let t = self.frame as u8;

        let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        context.set_fill_style(&color.into());
        context.fill_rect(0.0, 0.0, width as f64, height as f64);
    }

    fn on_debug_keydown(
        &mut self,
        event: &web_sys::KeyboardEvent,
        input: &web_sys::HtmlInputElement,
    ) {
        let key = event.key();
        let text = input.value();
        match key.as_str() {
            "Enter" => {
                log::info!("Command: {text}");

                self.cmd_buffer.remove(self.cmd_index);
                self.cmd_buffer.push_front(text);
                self.cmd_buffer.push_front("".to_string());
                self.cmd_index = 0;
                input.set_value("");
            }
            "Down" | "ArrowDown" => {
                self.cmd_buffer[self.cmd_index] = text;
                let new_index = self.cmd_index.saturating_sub(1);
                let new_index = new_index.clamp(0, self.cmd_buffer.len() - 1);
                self.cmd_index = new_index;
                let new_text = self.cmd_buffer[new_index].as_str();
                input.set_value(new_text);
            }
            "Up" | "ArrowUp" => {
                self.cmd_buffer[self.cmd_index] = text;
                let new_index = self.cmd_index.saturating_add(1);
                let new_index = new_index.clamp(0, self.cmd_buffer.len() - 1);
                self.cmd_index = new_index;
                let new_text = self.cmd_buffer[new_index].as_str();
                input.set_value(new_text);
            }
            _ => {}
        }
    }
}

pub fn app_main() {
    let app = MainApp::new();
    let sys = BaseSys::new(app, CANVAS_W, CANVAS_H);
    sys.start();
}
