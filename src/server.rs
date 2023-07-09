use crate::db;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{get, middleware, web, App, HttpServer, Responder, Result};
use futures::{join, TryFutureExt};
use rusqlite::Connection;
use std::path::PathBuf;

struct AppState {
    // app_dir: PathBuf,
    connection: Connection,
}

pub async fn run_server(working_dir: PathBuf, open: bool) -> Result<()> {
    log::info!("starting HTTP server at http://localhost:8080");

    let app_dir = PathBuf::from("static");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                connection: db::create_connection(&working_dir).unwrap(),
                // app_dir: PathBuf::from("static"),
            }))
            .wrap(Cors::permissive())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .service(get_projects)
                    .service(get_tags)
                    .service(get_contributors)
                    .service(get_project_warnings)
                    .service(get_project_dependencies)
                    .service(get_project_tests)
                    .service(get_angular_metadata)
                    .service(get_snapshots),
            )
            .service(Files::new("/", &app_dir).index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .unwrap_or_else(|err| log::error!("{:?}", err));

    let open = open_report(open).unwrap_or_else(|err| log::error!("{:?}", err));

    join!(server, open);
    Ok(())
}

#[get("/projects")]
async fn get_projects(data: web::Data<AppState>) -> Result<impl Responder> {
    let conn = &data.connection;
    let projects = db::get_projects(conn).unwrap();
    Ok(web::Json(projects))
}

#[get("/projects/{pid}/snapshots")]
async fn get_snapshots(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let pid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_project_snapshots(conn, pid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/projects/{pid}/tags")]
async fn get_tags(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let pid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_tags(conn, pid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/projects/{pid}/snapshots/{sid}/contributors")]
async fn get_contributors(
    path: web::Path<(i64, i64)>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let (pid, sid) = path.into_inner();
    let conn = &data.connection;
    let result = db::get_contributors(conn, pid, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/projects/{pid}/warnings")]
async fn get_project_warnings(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let pid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_project_warnings(conn, pid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/projects/{pid}/dependencies")]
async fn get_project_dependencies(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let pid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_project_dependencies(conn, pid).unwrap();
    Ok(web::Json(result))
}

#[get("/reports/{pid}/tests")]
async fn get_project_tests(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let pid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_tests_stats(conn, pid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/reports/{pid}/angular")]
async fn get_angular_metadata(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let pid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_angular_metadata(conn, pid).unwrap();
    Ok(web::Json(result))
}

async fn open_report(open: bool) -> std::io::Result<()> {
    if open {
        log::info!("Opening report");
        webbrowser::open("http://127.0.0.1:8080")
    } else {
        Ok(())
    }
}
