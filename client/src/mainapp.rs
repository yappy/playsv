use crate::basesys::BaseSys;

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

pub fn app_main() {
    let sys = BaseSys::new(CANVAS_W, CANVAS_H);
    sys.start();
}
