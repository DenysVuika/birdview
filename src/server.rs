use crate::db::TestKind;
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
            .service(
                web::scope("/api")
                    .service(list_projects)
                    .service(list_authors)
                    .service(list_warnings)
                    .service(list_unit_tests)
                    .service(list_e2e_tests),
            )
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

#[get("/snapshots/{id}/warnings")]
async fn list_warnings(
    path: web::Path<(i64)>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_warnings(conn, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/authors")]
async fn list_authors(path: web::Path<(i64)>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_authors(conn, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/unit-tests")]
async fn list_unit_tests(
    path: web::Path<(i64)>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_tests(conn, sid, TestKind::Unit).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/e2e-tests")]
async fn list_e2e_tests(
    path: web::Path<(i64)>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = match db::get_tests(conn, sid, TestKind::EndToEnd) {
        Ok(tests) => tests,
        Err(_) => vec![],
    };
    Ok(web::Json(result))
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

    let result_data = format!(
        "window.snapshotId=\"{}\"; window.data = {};",
        params.1, json_string
    );
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
