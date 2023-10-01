use crate::{
    asset,
    basesys::{App, BaseSys},
    jsif,
    net::PollingHttp,
};
use anyhow::{bail, Result};
use getopts::{Matches, Options};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

const SERVER: &str = "127.0.0.1:8888";

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

type DbgCmdFunc = dyn Fn(&mut MainApp, &Options, Matches) -> Result<()>;
struct DbgCmd {
    opts: Rc<Options>,
    func: Rc<Box<DbgCmdFunc>>,
}

enum State {
    Init,
    SelectRoom(Option<jsif::RoomList>),
    //Main(Option<jsif::LocalView>),
}

struct MainApp {
    state: Rc<RefCell<State>>,

    http: PollingHttp,
    frame: u64,
    fps: f64,
    fps_start: f64,
    fps_count: u64,
    server_info: Rc<RefCell<Option<String>>>,
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
            state: Rc::new(RefCell::new(State::Init)),

            http,
            frame: 0,
            fps: 0.0,
            fps_start: 0.0,
            fps_count: 0,
            server_info: Rc::new(RefCell::new(None)),
            testimg,
            dbg_cmds,
        }
    }
}

impl App for MainApp {
    fn init(&mut self) {
        let url = format!("http://{SERVER}/api/info");
        let dest = Rc::clone(&self.server_info);
        self.http.get(&url, move |result| {
            let info: jsif::ServerInfo = serde_json::from_str(result.expect("HTTP request error"))
                .expect("Json parse error");
            *dest.borrow_mut() = Some(format!("{}\n{}", info.version, info.description));
        });

        let url = format!("http://{SERVER}/api/room");
        let dest = Rc::clone(&self.state);
        self.http.get(&url, move |result| {
            let rooms: jsif::RoomList = serde_json::from_str(result.expect("HTTP request error"))
                .expect("Json parse error");
            *dest.borrow_mut() = State::SelectRoom(Some(rooms));
        });
        *self.state.borrow_mut() = State::SelectRoom(None);
    }

    fn frame(&mut self) {
        // fps
        let now = web_sys::window().unwrap().performance().unwrap().now();
        let elapsed = now - self.fps_start;
        self.fps_count += 1;
        if elapsed > 1000.0 {
            self.fps = self.fps_count as f64 / (elapsed / 1000.0);
            self.fps_count = 0;
            self.fps_start = now;
        }
        self.frame += 1;

        // poll network
        self.http.poll();
    }

    fn render(&mut self, context: &CanvasRenderingContext2d, width: u32, height: u32) {
        let t = self.frame as u8;
        context.save();

        // clear
        let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        context.set_fill_style(&color.into());
        context.fill_rect(0.0, 0.0, width as f64, height as f64);

        // server info
        context.set_fill_style(&"white".to_string().into());
        context.set_font("10px monospace");
        let info = &*self.server_info.borrow();
        let infostr = if let Some(info) = info {
            info
        } else {
            "Getting server info..."
        };
        context.fill_text(infostr, 10.0, 10.0).unwrap();

        // fps
        context
            .fill_text(&format!("{:0>5.2}", self.fps), width as f64 - 30.0, 10.0)
            .unwrap();

        context
            .draw_image_with_html_image_element(&self.testimg, 320.0, 240.0)
            .unwrap();

        context.restore();

        let state = &*self.state.borrow();
        match state {
            State::Init => {}
            State::SelectRoom(rooms) => {
                self.render_select_room(context, rooms);
            }
        }
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

impl MainApp {
    fn render_select_room(
        &self,
        context: &CanvasRenderingContext2d,
        rooms: &Option<jsif::RoomList>,
    ) {
        if rooms.is_none() {
            return;
        }
        let rooms = rooms.as_ref().unwrap();

        context.set_fill_style(&"white".to_string().into());
        context.set_font("32px serif");
        context
            .fill_text(&format!("{} Rooms", rooms.0.len()), 50.0, 50.0)
            .unwrap();
    }
}

impl MainApp {
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

    fn insert_dbg_cmd<F>(
        map: &mut HashMap<&str, DbgCmd>,
        name: &'static str,
        opts: Options,
        func: F,
    ) where
        F: Fn(&mut MainApp, &Options, Matches) -> Result<()> + 'static,
    {
        map.insert(
            name,
            DbgCmd {
                opts: Rc::new(opts),
                func: Rc::new(Box::new(func)),
            },
        );
    }

    fn create_dbg_cmds() -> HashMap<&'static str, DbgCmd> {
        let mut dbg_cmds = HashMap::new();

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        Self::insert_dbg_cmd(&mut dbg_cmds, "help", opts, Self::dbg_help);

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        Self::insert_dbg_cmd(&mut dbg_cmds, "file", opts, Self::dbg_file);

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        Self::insert_dbg_cmd(&mut dbg_cmds, "http", opts, Self::dbg_http);

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        opts.optopt("c", "create", "Create a room", "ROOM_COMMENT");
        Self::insert_dbg_cmd(&mut dbg_cmds, "room", opts, Self::dbg_room);

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
                let base64 = asset::read_file(name)?;
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
        self.http.get(url, |result| {
            log::debug!("{:?}", result);
        });

        Ok(())
    }

    fn dbg_room(&mut self, opts: &Options, args: Matches) -> Result<()> {
        if args.opt_present("h") {
            let brief = "Room API.\nroom [options] [URL]";
            log::debug!("{}", opts.usage(brief));
            return Ok(());
        }

        if let Some(comment) = args.opt_str("c") {
            let url = format!("http://{SERVER}/api/room");
            let param = jsif::CreateRoom {
                comment: comment.clone(),
            };
            self.http.post(&url, &param, |result| {
                log::debug!("{:?}", result);
                if let Ok(json) = result {
                    if let Ok(room) = serde_json::from_str::<jsif::Room>(json) {
                        log::debug!("Room created successfully: room_id={}", room.id);
                    }
                }
            });
        } else {
            let url = format!("http://{SERVER}/api/room");
            self.http.get(&url, |result| {
                log::debug!("{:?}", result);
            });
        }

        Ok(())
    }
}

pub fn app_main() {
    let app = MainApp::new();
    let sys = BaseSys::new(app, CANVAS_W, CANVAS_H);
    sys.start();
}
