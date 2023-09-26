use std::{collections::HashMap, rc::Rc};

use crate::basesys::{App, BaseSys};
use getopts::{Matches, Options};
use web_sys::CanvasRenderingContext2d;

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

type DbgCmdFunc = dyn Fn(&mut MainApp, Matches);
struct DbgCmd {
    opts: Options,
    func: Rc<Box<DbgCmdFunc>>,
}

struct MainApp {
    frame: u64,

    dbg_cmds: HashMap<&'static str, DbgCmd>,
}

impl MainApp {
    fn new() -> Self {
        let dbg_cmds = Self::create_dbg_cmds();

        Self { frame: 0, dbg_cmds }
    }

    fn exec_dbg_cmd(&mut self, cmd: &str, args: &str) {
        let func;
        let matches;
        if let Some(v) = self.dbg_cmds.get(cmd) {
            match v.opts.parse(args.split_ascii_whitespace()) {
                Ok(m) => {
                    func = v.func.clone();
                    matches = m;
                }
                Err(e) => {
                    log::error!("{e}");
                    return;
                }
            }
        } else {
            log::error!("Command not found: {cmd}");
            return;
        }
        func(self, matches);
    }

    fn create_dbg_cmds() -> HashMap<&'static str, DbgCmd> {
        let mut dbg_cmds = HashMap::new();

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        dbg_cmds.insert(
            "help",
            DbgCmd {
                opts,
                func: Rc::new(Box::new(Self::dbg_help)),
            },
        );

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        dbg_cmds.insert(
            "echo",
            DbgCmd {
                opts,
                func: Rc::new(Box::new(Self::dbg_echo)),
            },
        );

        dbg_cmds
    }

    fn dbg_help(&mut self, _args: Matches) {
        log::debug!("HELP OK");
    }
    fn dbg_echo(&mut self, _args: Matches) {}
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
        self.exec_dbg_cmd(cmd, args);
    }
}

pub fn app_main() {
    let app = MainApp::new();
    let sys = BaseSys::new(app, CANVAS_W, CANVAS_H);
    sys.start();
}
