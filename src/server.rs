use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use futures::{join, TryFutureExt};

pub async fn run_server(open: bool) -> Result<()> {
    println!("Starting web server at 127.0.0.1:8080");
    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .unwrap_or_else(|err| println!("{:?}", err));

    let open = open_report(open).unwrap_or_else(|err| println!("{:?}", err));

    join!(server, open);
    Ok(())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn open_report(open: bool) -> std::io::Result<()> {
    if open {
        println!("Opening report");
        webbrowser::open("http://127.0.0.1:8080")
    } else {
        Ok(())
    }
}
