use crate::db;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use futures::{join, TryFutureExt};
use rusqlite::Connection;
use std::path::PathBuf;

struct AppState {
    connection: Connection,
}

pub async fn run_server(working_dir: PathBuf, open: bool) -> Result<()> {
    log::info!("starting HTTP server at http://localhost:8080");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                connection: db::create_connection(&working_dir).unwrap(),
            }))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(get_projects)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .unwrap_or_else(|err| println!("{:?}", err));

    let open = open_report(open).unwrap_or_else(|err| println!("{:?}", err));

    join!(server, open);
    Ok(())
}

#[get("/api/projects")]
async fn get_projects(data: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    // let template = include_str!("assets/html/index.html");
    let conn = &data.connection;
    let projects = db::get_projects(conn).unwrap();
    Ok(web::Json(projects))

    // let data = format!("window.data = {};", json_string);
    // let template = template.replace("// <birdview:DATA>", &data);
    // HttpResponse::Ok().body(template)
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn open_report(open: bool) -> std::io::Result<()> {
    if open {
        log::info!("Opening report");
        webbrowser::open("http://127.0.0.1:8080")
    } else {
        Ok(())
    }
}
