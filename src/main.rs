use actix_files as fs;
use actix_web::{
    get, http::header, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use tracing::info;

#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Listening on http://localhost:8999/ ...");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(favicon)
            .service(fs::Files::new("/s", "static"))
            .service(web::resource("/").route(web::get().to(|| async {
                HttpResponse::Found()
                    .insert_header((header::LOCATION, "/s/index.html"))
                    .finish()
            })))
    })
    .bind("0.0.0.0:8999")?
    .run()
    .await
}
