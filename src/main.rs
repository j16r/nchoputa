extern crate actix_web;

use std::{env, io};

use actix_files as fs;
use actix_web::{
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Result,
    FromRequest,
    get,
    http::header,
    web,
};

// #[get("/favicon")]
// async fn favicon(_req: &HttpRequest) -> Result<fs::NamedFile> {
//     Ok(fs::NamedFile::open("static/favicon.ico")?)
// }

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    // env_logger::init();

    println!("Listening on http://localhost:8999/ ...");
    HttpServer::new(
        || App::new()
            .service(fs::Files::new("/s", "static"))
            // .service(favicon)
            .service(web::resource("/").route(web::get().to(|req: HttpRequest| {
                println!("{:?}", req);
                HttpResponse::Found()
                    .header(header::LOCATION, "/s/index.html")
                    .finish()
            })))
        )
        .bind("127.0.0.1:8999")?
        .run()
        .await
}
