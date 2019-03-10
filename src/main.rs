extern crate actix_web;
use actix_web::{server, App, http::Method, http::header, fs, HttpResponse};

fn main() {
    server::new(
        || App::new()
            .handler("/s", fs::StaticFiles::new("static").unwrap())
            .resource("/", |r| r.method(Method::GET).f(|req| {
                println!("{:?}", req);
                HttpResponse::Found()
                    .header(header::LOCATION, "/s/index.html")
                    .finish()
        }))
        )
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
