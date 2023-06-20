use crate::{db, report};
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder, Result};
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
            .service(web::scope("/api").service(list_projects))
            .service(index)
            .service(report_details)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .unwrap_or_else(|err| println!("{:?}", err));

    let open = open_report(open).unwrap_or_else(|err| println!("{:?}", err));

    join!(server, open);
    Ok(())
}

#[get("/projects")]
async fn list_projects(data: web::Data<AppState>) -> Result<impl Responder> {
    let conn = &data.connection;
    let projects = db::get_projects(conn).unwrap();
    Ok(web::Json(projects))
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/projects/{project}/{snapshot}")]
async fn report_details(
    path: web::Path<(String, i64)>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let params = path.into_inner();
    let conn = &data.connection;
    let template = include_str!("assets/html/index.html");
    let data = report::generate_report(conn, params.1).unwrap();
    let json_string = serde_json::to_string_pretty(&data)?;

    let result_data = format!("window.data = {};", json_string);
    let result_template = template.replace("// <birdview:DATA>", &result_data);

    Ok(HttpResponse::Ok().body(result_template))
}

async fn open_report(open: bool) -> std::io::Result<()> {
    if open {
        log::info!("Opening report");
        webbrowser::open("http://127.0.0.1:8080")
    } else {
        Ok(())
    }
}
