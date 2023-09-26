use crate::basesys::{App, BaseSys};
use web_sys::CanvasRenderingContext2d;

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

struct MainApp {
    frame: u64,
}

impl MainApp {
    fn new() -> Self {
        Self { frame: 0 }
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

    fn on_debug_command(&mut self, cmdline: &str) {
        let idx = cmdline.find(' ');
        let (cmd, args) = if let Some(idx) = idx {
            (&cmdline[..idx], &cmdline[idx + 1..])
        } else {
            (cmdline, "")
        };
        log::info!("cmd: {cmd}, args: {args}");
    }
}

pub fn app_main() {
    let app = MainApp::new();
    let sys = BaseSys::new(app, CANVAS_W, CANVAS_H);
    sys.start();
}
