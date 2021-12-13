mod mj;
mod mjsys;

use git_version::git_version;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use serde_json;
use std::sync::Mutex;


// from build system
const GIT_VERSION: &str = git_version!();

// shared with all App threads
struct AppState {
    test_data: Mutex<Vec<i32>>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

#[get("/test")]
async fn test(data: web::Data<AppState>) -> impl Responder {
    let result;
    {
        let mut test_data = data.test_data.lock().unwrap();
        test_data.push(4);
        result = format!("{:?}", test_data);
        // unlock
    }

    HttpResponse::Ok().body("result")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create shared state object (Arc internally)
    let app_state = web::Data::new(AppState {
        test_data: Mutex::new(vec![1, 2, 3]),
    });

    // pass a function as App builder
    // move app_state into closure
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(info)
            .service(test)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
