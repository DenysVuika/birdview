use crate::db;
use crate::db::{NgKind, TestKind};
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder, Result};
use futures::{join, TryFutureExt};
use rusqlite::Connection;
use serde_json::json;
use std::collections::HashMap;
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
                    .service(get_angular_metadata)
                    .service(get_snapshots)
                    .service(get_angular)
                    .service(get_snapshot_project)
                    .service(get_authors)
                    .service(get_warnings)
                    .service(get_packages)
                    .service(get_dependencies)
                    .service(get_unit_tests)
                    .service(get_e2e_tests)
                    .service(get_file_types),
            )
            .service(view_snapshot_details)
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

#[deprecated]
#[get("/snapshots/{id}/angular")]
async fn get_angular(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;

    let ng_version = "unknown";
    let modules = db::get_ng_entities(conn, sid, NgKind::Module).unwrap_or(vec![]);
    let components = db::get_ng_entities(conn, sid, NgKind::Component).unwrap_or(vec![]);
    let directives = db::get_ng_entities(conn, sid, NgKind::Directive).unwrap_or(vec![]);
    let services = db::get_ng_entities(conn, sid, NgKind::Service).unwrap_or(vec![]);
    let pipes = db::get_ng_entities(conn, sid, NgKind::Pipe).unwrap_or(vec![]);
    let dialogs = db::get_ng_entities(conn, sid, NgKind::Dialog).unwrap_or(vec![]);

    let angular = json!({
        "version": ng_version,
        "modules": modules,
        "components": components,
        "directives": directives,
        "services": services,
        "pipes": pipes,
        "dialogs": dialogs
    });
    Ok(web::Json(angular))
}

#[deprecated]
#[get("/snapshots/{id}/project")]
async fn get_snapshot_project(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let snapshot = db::get_snapshot_by_id(conn, sid).unwrap();
    let project = db::get_project_by_snapshot(conn, sid).unwrap();
    let result = json!({
        "name": project.name,
        "created_on": snapshot.created_on,
        "origin": project.origin,
        "tag": snapshot.tag,
        "sha": snapshot.sha,
        "timestamp": snapshot.timestamp
    });
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/warnings")]
async fn get_warnings(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_warnings(conn, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/authors")]
async fn get_authors(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_authors(conn, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/file-types")]
async fn get_file_types(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_file_types(conn, sid).unwrap_or(HashMap::new());
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/packages")]
async fn get_packages(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_packages(conn, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/dependencies")]
async fn get_dependencies(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_dependencies(conn, sid).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/unit-tests")]
async fn get_unit_tests(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = db::get_tests(conn, sid, TestKind::Unit).unwrap_or(vec![]);
    Ok(web::Json(result))
}

#[get("/snapshots/{id}/e2e-tests")]
async fn get_e2e_tests(path: web::Path<i64>, data: web::Data<AppState>) -> Result<impl Responder> {
    let sid = path.into_inner();
    let conn = &data.connection;
    let result = match db::get_tests(conn, sid, TestKind::EndToEnd) {
        Ok(tests) => tests,
        Err(_) => vec![],
    };
    Ok(web::Json(result))
}

#[get("/snapshots/{snapshot}")]
async fn view_snapshot_details(path: web::Path<i64>) -> Result<HttpResponse> {
    let sid = path.into_inner();
    let template = include_str!("../static/index.html");
    let result_data = format!("window.snapshotId=\"{}\";", sid);
    let result_template = template.replace("// <birdview:DATA>", &result_data);

    Ok(HttpResponse::Ok().body(result_template))
}

async fn open_report(open: bool) -> std::io::Result<()> {
    if open {
        log::info!("Opening report");
        webbrowser::open("http://127.0.0.1:8080/snapshots/1")
    } else {
        Ok(())
    }
}
