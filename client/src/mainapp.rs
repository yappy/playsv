use std::{collections::HashMap, rc::Rc};

use crate::basesys::{App, BaseSys};
use anyhow::{bail, Result};
use getopts::{Matches, Options};
use web_sys::CanvasRenderingContext2d;

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

type DbgCmdFunc = dyn Fn(&mut MainApp, &Options, Matches) -> Result<()>;
struct DbgCmd {
    opts: Rc<Options>,
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

    fn exec_dbg_cmd(&mut self, cmd: &str, args: &str) -> Result<()> {
        let (func, opts, matches);
        if let Some(v) = self.dbg_cmds.get(cmd) {
            match v.opts.parse(args.split_ascii_whitespace()) {
                Ok(m) => {
                    func = v.func.clone();
                    opts = v.opts.clone();
                    matches = m;
                }
                Err(e) => {
                    bail!(e);
                }
            }
        } else {
            bail!("Command not found: {cmd}");
        }

        func(self, &opts, matches)
    }

    fn create_dbg_cmds() -> HashMap<&'static str, DbgCmd> {
        let mut dbg_cmds = HashMap::new();

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        dbg_cmds.insert(
            "help",
            DbgCmd {
                opts: Rc::new(opts),
                func: Rc::new(Box::new(Self::dbg_help)),
            },
        );

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        dbg_cmds.insert(
            "frame",
            DbgCmd {
                opts: Rc::new(opts),
                func: Rc::new(Box::new(Self::dbg_frame)),
            },
        );

        dbg_cmds
    }

    fn dbg_help(&mut self, opts: &Options, args: Matches) -> Result<()> {
        if args.opt_present("h") {
            let brief = "Print help for COMMAND.\nhelp [options] [COMMAND...]";
            log::debug!("{}", opts.usage(brief));
            return Ok(());
        }

        if args.free.is_empty() {
            let mut buf = String::new();
            for (&k, v) in self.dbg_cmds.iter() {
                buf += "\n";
                buf += &v.opts.short_usage(k);
            }
            log::debug!("Command List{buf}");
        } else {
            for cmd in args.free.iter() {
                let (func, opts, matches);
                if let Some(v) = self.dbg_cmds.get(cmd.as_str()) {
                    func = v.func.clone();
                    opts = v.opts.clone();
                    matches = v.opts.parse(["-h"])?;
                } else {
                    bail!("Command not found: {cmd}");
                };
                func(self, &opts, matches)?;
            }
        }

        Ok(())
    }
    fn dbg_frame(&mut self, opts: &Options, args: Matches) -> Result<()> {
        if args.opt_present("h") {
            let brief = "Get/Set frame count.\nframe [options] [SETVALUE]";
            log::debug!("{}", opts.usage(brief));
            return Ok(());
        }

        if !args.free.is_empty() {
            let value = args.free[0].parse()?;
            self.frame = value;
            log::debug!("Set: {}", value);
        } else {
            log::debug!("{}", self.frame);
        }

        Ok(())
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

        match self.exec_dbg_cmd(cmd, args) {
            Ok(()) => {}
            Err(e) => {
                log::error!("{:?}", e)
            }
        }
    }
}

pub fn app_main() {
    let app = MainApp::new();
    let sys = BaseSys::new(app, CANVAS_W, CANVAS_H);
    sys.start();
}
