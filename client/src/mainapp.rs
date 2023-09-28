use crate::{
    asset,
    basesys::{App, BaseSys},
    net::PollingHttp,
};
use anyhow::{bail, Result};
use getopts::{Matches, Options};
use std::{collections::HashMap, rc::Rc};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

type DbgCmdFunc = dyn Fn(&mut MainApp, &Options, Matches) -> Result<()>;
struct DbgCmd {
    opts: Rc<Options>,
    func: Rc<Box<DbgCmdFunc>>,
}

struct MainApp {
    http: PollingHttp,
    frame: u64,
    testimg: HtmlImageElement,

    dbg_cmds: HashMap<&'static str, DbgCmd>,
}

impl MainApp {
    fn new() -> Self {
        let dbg_cmds = Self::create_dbg_cmds();

        let http = PollingHttp::new();
        let testimg = HtmlImageElement::new().unwrap();
        let testdata = format!(
            "data:image/gif;base64,{}",
            asset::read_file("manzu0/p_ms1_0.gif").unwrap()
        );
        testimg.set_src(&testdata);

        Self {
            http,
            frame: 0,
            testimg,
            dbg_cmds,
        }
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
        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        dbg_cmds.insert(
            "file",
            DbgCmd {
                opts: Rc::new(opts),
                func: Rc::new(Box::new(Self::dbg_file)),
            },
        );
        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        dbg_cmds.insert(
            "http",
            DbgCmd {
                opts: Rc::new(opts),
                func: Rc::new(Box::new(Self::dbg_http)),
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

    fn dbg_file(&mut self, opts: &Options, args: Matches) -> Result<()> {
        if args.opt_present("h") {
            let brief = "Show files.\nfiles [options]";
            log::debug!("{}", opts.usage(brief));
            return Ok(());
        }

        if args.free.is_empty() {
            log::debug!("All files:\n{}", asset::get_file_list().join("\n"));
        } else {
            for name in args.free.iter() {
                let base64 = asset::read_file(&name)?;
                log::debug!("{name} {}\n{}", base64.len(), base64)
            }
        }

        Ok(())
    }

    fn dbg_http(&mut self, opts: &Options, args: Matches) -> Result<()> {
        if args.opt_present("h") {
            let brief = "HTTP request.\nhttp [options] [URL]";
            log::debug!("{}", opts.usage(brief));
            return Ok(());
        }

        let url = args
            .free
            .get(0)
            .map_or("http://127.0.0.1:8080/", |s| s.as_str());
        log::debug!("HTTP: {url}");
        self.http.request(url, |result| {
            log::debug!("{:?}", result);
        });

        Ok(())
    }
}

impl App for MainApp {
    fn frame(&mut self) {
        self.http.poll();
        self.frame += 1;
    }

    fn render(&mut self, context: &CanvasRenderingContext2d, width: u32, height: u32) {
        let t = self.frame as u8;

        let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        context.set_fill_style(&color.into());
        context.fill_rect(0.0, 0.0, width as f64, height as f64);

        context.draw_image_with_html_image_element(&self.testimg, 320.0, 240.0).unwrap();
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
