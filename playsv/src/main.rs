mod mj;
mod mjsys;

use git_version::git_version;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize};
use serde_json;
use std::sync::RwLock;


// from build system
const GIT_VERSION: &str = git_version!();

// shared with all App threads
struct AppState {
    games: RwLock<Vec<mj::GameState>>,
}

fn simple_html(title: &str, body: &str) -> String {
    format!(r#"
<!DOCTYPE html>
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
        title, body)
}

#[get("/info")]
async fn info() -> impl Responder {
    #[derive(Serialize)]
    struct InfoObj {
        description: String,
        version: String,
    }

    let info = InfoObj {
        description: "now testing...".to_string(),
        version: GIT_VERSION.to_string(),
    };
    let body = serde_json::to_string(&info).unwrap();

    HttpResponse::Ok().content_type("application/json").body(body)
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let len;
    {
        let games = data.games.read().unwrap();
        len = games.len();
        // unlock
    }

    HttpResponse::Ok().body(simple_html("Hello",
        &format!{"There is/are {} game(s).", len}))
}

#[get("/create")]
async fn create(data: web::Data<AppState>) -> impl Responder {
    {
        let mut games = data.games.write().unwrap();
        games.push(mj::GameState::new());
        // unlock
    }

    HttpResponse::Ok().body(r#"<meta http-equiv="refresh" content="0;URL=./">"#)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create shared state object (Arc internally)
    let app_state = web::Data::new(AppState {
        games: RwLock::new(vec![]),
    });

    // pass a function as App builder
    // move app_state into closure
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(info)
            .service(create)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
