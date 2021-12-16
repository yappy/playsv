mod mj;
mod mjsys;

use git_version::git_version;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;


// from build system
const GIT_VERSION: &str = git_version!();

// description, message, etc.
const STRING_MAX: usize = 1024;

// shared with all App threads
struct AppState {
    // next game room id to be created
    next_id: AtomicU64,
    // (id -> Game, comment) sorted list
    games: RwLock<BTreeMap<u64, (mj::Game, String)>>,
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
    let mut msg = "".to_string();
    {
        // rlock
        let games = data.games.read().unwrap();
        for (k, _v) in &*games {
            msg.push_str(&format!("<p>{}</p>\n", k));
        }
        // unlock
    }

    HttpResponse::Ok().body(simple_html("Hello", &msg))
}

/*
 * URL: /game
 * Get/Add-to active game list
 */
#[derive(Serialize, Deserialize)]
struct GetGameResultElement {
    id: u64,
    comment: String,
}

#[derive(Serialize, Deserialize)]
struct GetGameResult {
    games: Vec<GetGameResultElement>,
}

#[get("/games")]
async fn get_games(data: web::Data<AppState>) -> impl Responder {
    let result;
    {
        // rlock
        let games = data.games.read().unwrap();
        let list = games.iter().map(
            |(id, (_game, comment))|
                GetGameResultElement{id: *id, comment: comment.clone()}
            ).collect();
        result = GetGameResult{games: list};
        // unlock
    }
    let body = serde_json::to_string(&result).unwrap();

    HttpResponse::Ok().content_type("application/json").body(body)
}

#[derive(Debug, Serialize, Deserialize)]
struct PostGameParam {
    //game_type: String,
    comment: String,
}
#[derive(Serialize, Deserialize)]
struct PostGameResult {
    id: u64,
}

// curl -X POST -H "Content-Type: application/json" -d '{"comment": "aaa"}' -v localhost:8080/games
#[post("/games")]
async fn post_games(data: web::Data<AppState>, param: web::Json<PostGameParam>) -> impl Responder {
    println!("POST /game {:?}", param);
    if param.comment.len() > STRING_MAX {
        return HttpResponse::BadRequest().finish();
    }

    // create a new game state
    let new_game = match mj::Game::new() {
        Some(game) => game,
        None => {return HttpResponse::BadRequest().finish();}
    };

    let id;
    {
        // wlock
        let mut games = data.games.write().unwrap();
        // load next id and increment atomically
        id = data.next_id.fetch_add(1, Ordering::Relaxed);
        games.insert(id, (new_game, param.comment.clone()));
        // unlock
    }

    let result = PostGameResult{id};
    let body = serde_json::to_string(&result).unwrap();

    HttpResponse::Ok().content_type("application/json").body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create shared state object (Arc internally)
    let app_state = web::Data::new(AppState {
        next_id: AtomicU64::new(1),
        games: RwLock::new(BTreeMap::new()),
    });

    // pass a function as App builder
    // move app_state into closure
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(info)
            .service(get_games)
            .service(post_games)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn simple_html(title: &str, body: &str) -> String {
    format!(r#"<!DOCTYPE html>
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
