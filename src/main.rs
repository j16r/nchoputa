use std::collections::HashMap;

use actix_files as fs;
use actix_web::{
    error, get, http::header, middleware, web, App, HttpResponse, HttpServer, Responder, Result,
};
use chrono::NaiveDate;
use postcard::to_allocvec;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, error};

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
        .insert("CSIRO".to_string(), "/api/graphs/csiro".to_string());
    graph_list
        .graphs
        .insert("UHSLC".to_string(), "/api/graphs/uhslc".to_string());
    to_allocvec(&graph_list).unwrap()
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Graph {
    points: Vec<(NaiveDate, f32)>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Row {
    Date: NaiveDate,
    Value: f32,
}

#[get("/api/graphs/{name}")]
async fn show_graph(name: web::Path<String>) -> Result<impl Responder> {
    match name.as_str() {
        "csiro" => {
            let mut rdr =
                csv::ReaderBuilder::new()
                    .delimiter(b'\t')
                    .from_reader(include_str!("../data/sealevel/csiro.tsv").as_bytes());
            let mut graph = Graph { points: Vec::new() };
            for result in rdr.deserialize() {
                let record: Row = result
                    .map_err(|e| {
                        error!("error reading dataset {}: {}", name, e);
                        error::ErrorInternalServerError("error reading source data")
                    })?;
                debug!("record: {:?}", record);
                graph.points.push((record.Date, record.Value));
            }
            Ok(to_allocvec(&graph).map_err(|e| {
                error!("error encoding dataset {}: {}", name, e);
                error::ErrorInternalServerError("error encoding dataset")
            })?)
            // Ok(include_str!("../data/sealevel/csiro.tsv"))
        }
        "uhslc" => {
            let mut rdr =
                csv::ReaderBuilder::new()
                    .delimiter(b'\t')
                    .from_reader(include_str!("../data/sealevel/uhslc.tsv").as_bytes());
            let mut graph = Graph { points: Vec::new() };
            for result in rdr.deserialize() {
                let record: Row = result
                    .map_err(|e| {
                        error!("error reading dataset {}: {}", name, e);
                        error::ErrorInternalServerError("error reading source data")
                    })?;
                debug!("record: {:?}", record);
                graph.points.push((record.Date, record.Value));
            }
            Ok(to_allocvec(&graph).map_err(|e| {
                error!("error encoding dataset {}: {}", name, e);
                error::ErrorInternalServerError("error encoding dataset")
            })?)
        }
        _ => Err(error::ErrorNotFound(format!("no graph with name {}", name))),
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
