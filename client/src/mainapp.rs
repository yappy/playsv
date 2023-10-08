use crate::{
    asset,
    basesys::{App, BaseSys},
    net::PollingHttp,
    testmode::{self, TestMode},
};
use anyhow::{bail, Result};
use game::{jsif, mjsys};
use getopts::{Matches, Options};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

// On server build, TRUNK_BUILD_PUBLIC_URL will be specified by env
const APIROOT: Option<&str> = option_env!("TRUNK_BUILD_PUBLIC_URL");

fn apiroot() -> &'static str {
    APIROOT.unwrap_or("/")
}

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

type DbgCmdFunc = dyn Fn(&mut MainApp, &Options, Matches) -> Result<()>;
struct DbgCmd {
    opts: Rc<Options>,
    func: Rc<Box<DbgCmdFunc>>,
}

enum State {
    Init,
    TestMode,
    SelectRoom(Option<Box<jsif::RoomList>>),
    Main(Option<Box<jsif::LocalView>>),
}

#[derive(Default)]
pub struct ImageSet {
    // [kind][num]
    pub pai: [Vec<HtmlImageElement>; 4],
}

pub struct HitBox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl HitBox {
    pub fn from_image(img: &HtmlImageElement, x: u32, y: u32) -> Self {
        Self {
            x: x as i32,
            y: y as i32,
            w: img.width() as i32,
            h: img.height() as i32,
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

struct MainApp {
    state: Rc<RefCell<State>>,

    http: PollingHttp,
    frame: u64,
    fps: f64,
    fps_start: f64,
    fps_count: u64,
    server_info: Rc<RefCell<Option<String>>>,

    img_set: Rc<ImageSet>,

    test_mode: testmode::TestMode,

    dbg_cmds: HashMap<&'static str, DbgCmd>,
}

impl MainApp {
    fn new() -> Self {
        let dbg_cmds = Self::create_dbg_cmds();

        let http = PollingHttp::new();

        let mut img_set: ImageSet = Default::default();
        let kind_table = ["ms", "ps", "ss"];
        let zu_table = ["", "ji_e", "ji_s", "ji_w", "ji_n", "no", "ji_h", "ji_c"];
        for (kind, &mut ref mut list) in img_set.pai.iter_mut().enumerate() {
            let is_zu = kind == 3;
            let maxnum = if is_zu { 7 } else { 9 };

            #[allow(clippy::needless_range_loop)]
            for num in 1..=maxnum {
                let fname = if !is_zu {
                    format!("pai/p_{}{}_0.gif", kind_table[kind], num)
                } else {
                    format!("pai/p_{}_0.gif", zu_table[num])
                };
                let img = HtmlImageElement::new().unwrap();
                let testdata = format!(
                    "data:image/gif;base64,{}",
                    asset::read_file(&fname).unwrap_or_else(|e| panic!("{e}"))
                );
                img.set_src(&testdata);

                list.push(img);
            }
        }
        let img_set = Rc::new(img_set);

        //let initial_state = State::Init;
        let initial_state = State::TestMode;
        let state = Rc::new(RefCell::new(initial_state));

        let test_mode = TestMode::new(Rc::clone(&img_set));

        Self {
            state,

            http,
            frame: 0,
            fps: 0.0,
            fps_start: 0.0,
            fps_count: 0,
            server_info: Rc::new(RefCell::new(None)),

            img_set,

            test_mode,

            dbg_cmds,
        }
    }
}

impl App for MainApp {
    fn init(&mut self) {
        // Get room data and go to SelectRoom state.
        /*
        let url = format!("{}api/info", apiroot());
        let dest = Rc::clone(&self.server_info);
        self.http.get(&url, move |result| {
            let info: jsif::ServerInfo = serde_json::from_str(result.expect("HTTP request error"))
                .expect("Json parse error");
            *dest.borrow_mut() = Some(format!("{}\n{}", info.version, info.description));
        });

        let url = format!("{}api/room", apiroot());
        let dest = Rc::clone(&self.state);
        self.http.get(&url, move |result| {
            let rooms: jsif::RoomList = serde_json::from_str(result.expect("HTTP request error"))
                .expect("Json parse error");
            *dest.borrow_mut() = State::SelectRoom(Some(Box::new(rooms)));
        });
        *self.state.borrow_mut() = State::SelectRoom(None);
        */
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
        context.save();

        // clear
        //let color = format!("#{0:>02x}{0:>02x}{0:>02x}", t);
        let color = "darkgreen";
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

        context.restore();

        let state = &*self.state.borrow();
        match state {
            State::Init => {}
            State::TestMode => {
                self.test_mode.render(context, width, height);
            }
            State::SelectRoom(rooms) => {
                self.render_select_room(context, rooms);
            }
            State::Main(view) => {
                self.render_game_main(context, view);
            }
        }
    }

    fn on_mouse_click(&mut self, event: &web_sys::MouseEvent) {
        let state = &*self.state.borrow();
        match state {
            State::TestMode => {
                self.test_mode.click(event.x(), event.y());
            }
            _ => {}
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
        rooms: &Option<Box<jsif::RoomList>>,
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

    fn render_game_main(
        &self,
        context: &CanvasRenderingContext2d,
        view: &Option<Box<jsif::LocalView>>,
    ) {
        if view.is_none() {
            return;
        }

        let view = view.as_ref().unwrap();
        let mut x = 100.0;

        log::debug!("{:?}", view.local.hands[0]);
        for &pai in view.local.hands[0].iter() {
            let (kind, num) = mjsys::decode(pai as u8).unwrap();

            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            let w = img.width() as f64;
            context
                .draw_image_with_html_image_element(img, x, 250.0)
                .unwrap();
            x += w;
        }
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

        let mut opts = Options::new();
        opts.optflag("h", "help", "Print help");
        opts.optopt("r", "room", "Room ID", "ROOM_ID");
        opts.optopt("p", "player", "Create a room", "PLAYER#");
        Self::insert_dbg_cmd(&mut dbg_cmds, "game", opts, Self::dbg_game);

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
            let url = format!("{}api/room", apiroot());
            let param = jsif::CreateRoom {
                comment: comment.clone(),
            };
            self.http.post(&url, &param, |result| {
                log::debug!("{:?}", result);
                if let Ok(json) = result {
                    if let Ok(room) = serde_json::from_str::<jsif::Room>(json) {
                        log::debug!("Room created successfully\n{:?}", room);
                    }
                }
            });
        } else {
            let url = format!("{}api/room", apiroot());
            self.http.get(&url, |result| {
                log::debug!("{:?}", result);
                if let Ok(json) = result {
                    if let Ok(rooms) = serde_json::from_str::<jsif::RoomList>(json) {
                        log::debug!("{:?}", rooms);
                    }
                }
            });
        }

        Ok(())
    }

    fn dbg_game(&mut self, opts: &Options, args: Matches) -> Result<()> {
        if args.opt_present("h") {
            let brief = "Game API.\ngame [options]";
            log::debug!("{}", opts.usage(brief));
            return Ok(());
        }

        let room = args.opt_str("r").unwrap_or("0".to_string());
        let player = args.opt_str("p").unwrap_or("0".to_string());
        let url = format!("{}api/room/{room}/{player}", apiroot());
        let state = Rc::clone(&self.state);
        self.http.get(&url, move |result| {
            log::debug!("{:?}", result);
            if let Ok(json) = result {
                if let Ok(view) = serde_json::from_str::<jsif::LocalView>(json) {
                    log::debug!("{:?}", view);
                    *state.borrow_mut() = State::Main(Some(Box::new(view)));
                }
            }
        });

        Ok(())
    }
}

pub fn app_main() {
    let app = MainApp::new();
    let sys = BaseSys::new(app, CANVAS_W, CANVAS_H);
    sys.start();
}
