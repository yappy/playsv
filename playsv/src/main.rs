mod server;
mod mjgame;

use anyhow::Result;
use actix_web;

#[actix_web::main]
async fn main() -> Result<()> {
    server::server_main().await
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
