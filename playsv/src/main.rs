mod mjgame;
mod server;

use anyhow::Result;
use getopts::{Matches, Options};
use std::env;

const PORT_DEFAULT: u16 = 8888;

fn parse_options() -> Result<Matches> {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help");
    opts.optopt("p", "port", "Port number", "PORT");
    opts.optflag("c", "cors", "Enable CORS");

    let m = opts.parse(&args[1..])?;
    if m.opt_present("h") {
        let brief = format!("Usage: {program} [options]");
        print!("{}", opts.usage(&brief));
        std::process::exit(0);
    }

    Ok(m)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let m = parse_options()?;
    let port: u16 = m.opt_get_default("p", PORT_DEFAULT)?;
    let cors = m.opt_present("c");

    server::server_main(port, cors).await
}

/*
fn simple_html(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8"/>
<title>{}</title>
</head>
<body>
{}
</body>
</html>
"#,
        title, body
    )
}
*/
