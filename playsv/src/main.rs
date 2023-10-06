mod mjgame;

use actix_cors::Cors;
use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};
use common::jsif;
use git_version::git_version;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

// from build system
const GIT_VERSION: &str = git_version!();

const PUBLIC_URL: &str = env!("PUBLIC_URL");

// description, message, etc.
const STRING_MAX: usize = 1024;

// shared with all App threads
struct AppState {
    // next game room id to be created
    next_id: AtomicU64,
    // (id -> Game, comment) sorted list
    rooms: RwLock<BTreeMap<u64, (mjgame::Game, String)>>,
}

/// /info
///
/// jsif::ServerInfo
#[get("/api/info")]
async fn info() -> impl Responder {
    let info = jsif::ServerInfo {
        version: GIT_VERSION.to_string(),
        description: "This is a message from the server.".to_string(),
    };

    HttpResponse::Ok().json(info)
}

#[get("/")]
async fn root() -> impl Responder {
    file_serve("index.html").await
}

#[get("/{name}")]
async fn index(path: web::Path<String>) -> impl Responder {
    file_serve(&path).await
}

async fn file_serve(name: &str) -> impl Responder {
    const INDEX: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));
    const JS: &str = include_str!(concat!(env!("OUT_DIR"), "/client.js"));
    const WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/client_bg.wasm"));

    println!("GET /{name}");
    match name {
        "index.html" => HttpResponse::Ok().content_type("text/html").body(INDEX),
        "client.js" => HttpResponse::Ok().content_type("text/javascript").body(JS),
        "client_bg.wasm" => HttpResponse::Ok()
            .content_type("application/wasm")
            .body(WASM),
        _ => HttpResponse::NotFound().body(""),
    }
}

// Get/Add-to active game list
#[get("/api/room")]
async fn get_rooms(data: web::Data<AppState>) -> impl Responder {
    let result;
    {
        // rlock
        let rooms = data.rooms.read().unwrap();
        let list = rooms
            .iter()
            .map(|(id, (_game, comment))| jsif::Room {
                id: *id,
                comment: comment.clone(),
            })
            .collect();
        result = jsif::RoomList(list);
        // unlock
    }

    HttpResponse::Ok().json(result)
}

// curl -X POST -H "Content-Type: application/json" -d '{"comment": "aaa"}' -v localhost:8888/room
#[post("/api/room")]
async fn post_room(
    data: web::Data<AppState>,
    param: web::Json<jsif::CreateRoom>,
) -> impl Responder {
    println!("POST /api/rooms {:?}", param);
    if param.comment.len() > STRING_MAX {
        return HttpResponse::BadRequest().finish();
    }

    // create a new game state
    let new_game = match mjgame::Game::new() {
        Ok(game) => game,
        Err(err) => {
            return HttpResponse::BadRequest().json(jsif::ErrorMsg::new(err.to_string()));
        }
    };

    let (id, comment) = {
        // wlock
        let mut rooms = data.rooms.write().unwrap();
        // load next id and increment atomically
        let id = data.next_id.fetch_add(1, Ordering::Relaxed);
        rooms.insert(id, (new_game, param.comment.clone()));

        (id, param.comment.clone())
        // unlock
    };

    let result = jsif::Room { id, comment };
    let body = serde_json::to_string(&result).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
}

#[get("/api/room/{id}/{player}")]
async fn get_room_id_player(
    data: web::Data<AppState>,
    path: web::Path<(u64, u32)>,
) -> impl Responder {
    let (id, player) = path.into_inner();

    {
        // rlock game list
        let games = data.rooms.read().unwrap();
        let game = games.get(&id);
        if let Some(game) = game {
            let view = game.0.get_view(player);
            match view {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(err) => HttpResponse::BadRequest().json(jsif::ErrorMsg::new(err.to_string())),
            }
        } else {
            HttpResponse::BadRequest().json(jsif::ErrorMsg::new("Invalid id".to_string()))
        }
        // unlock
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create shared state object (Arc internally)
    let app_state = web::Data::new(AppState {
        next_id: AtomicU64::new(0),
        rooms: Default::default(),
    });

    println!("http://127.0.0.1:8888{PUBLIC_URL}/");
    // pass a function as App builder
    // move app_state into closure
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        App::new().wrap(cors).app_data(app_state.clone()).service(
            web::scope(PUBLIC_URL)
                .service(root)
                .service(index)
                .service(info)
                .service(get_rooms)
                .service(post_room)
                .service(get_room_id_player),
        )
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await
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
