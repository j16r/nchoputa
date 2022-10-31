use std::collections::HashMap;

use actix_files as fs;
use actix_web::{
    error, get, http::header, middleware, web, App, HttpResponse, HttpServer, Responder, Result,
};
use postcard::to_allocvec;
use serde::{Deserialize, Serialize};
use tracing::info;

#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct GraphList {
    graphs: HashMap<String, String>,
}

#[get("/api/graphs")]
async fn list_graphs() -> impl Responder {
    let mut graph_list = GraphList {
        graphs: HashMap::new(),
    };
    graph_list
        .graphs
        .insert("csiro".to_string(), "/api/graphs/csiro".to_string());
    graph_list
        .graphs
        .insert("uhslc".to_string(), "/api/graphs/uhslc".to_string());
    to_allocvec(&graph_list).unwrap()
}

#[get("/api/graphs/{name}")]
async fn show_graph(name: web::Path<String>) -> Result<impl Responder> {
    match name.as_str() {
        "csiro.tsv" => Ok(include_str!("../data/sealevel/csiro.tsv")),
        "uhslc.tsv" => Ok(include_str!("../data/sealevel/csiro.tsv")),
        _ => Err(error::ErrorNotFound(format!("no graph {}", name))),
    }
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
            .service(list_graphs)
            .service(show_graph)
    })
    .bind("0.0.0.0:8999")?
    .run()
    .await
}
