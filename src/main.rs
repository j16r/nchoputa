extern crate actix_web;
use actix_web::{
    App,
    fs,
    http::header,
    http::Method,
    HttpRequest,
    HttpResponse,
    Result,
    server,
};

fn favicon(_req: &HttpRequest) -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

fn main() {
    server::new(
        || App::new()
            .handler("/s", fs::StaticFiles::new("static").unwrap())
            .resource("/favicon.ico", |r| r.f(favicon))
            .resource("/", |r| r.method(Method::GET).f(|req| {
                println!("{:?}", req);
                HttpResponse::Found()
                    .header(header::LOCATION, "/s/index.html")
                    .finish()
        }))
        )
        .bind("127.0.0.1:8999").unwrap()
        .run();
}
