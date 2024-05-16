use std::collections::HashMap;

use actix_files as fs;
use actix_web::{
    error, get, http::header, middleware, web, App, HttpResponse, HttpServer, Responder, Result,
};
use chrono::NaiveDate;
use clap::Parser;
use postcard::to_allocvec;
use serde::Deserialize;
use shared::response::{Graph, GraphList};
use tracing::{debug, error, info};

#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[get("/api/graphs")]
async fn list_graphs() -> impl Responder {
    let mut graph_list = GraphList {
        graphs: HashMap::new(),
    };
    graph_list
        .graphs
        .insert("CSIRO".to_string(), "/api/graphs/CSIRO".to_string());
    graph_list
        .graphs
        .insert("UHSLC".to_string(), "/api/graphs/UHSLC".to_string());
    graph_list
        .graphs
        .insert("Dev".to_string(), "/api/graphs/Dev".to_string());
    to_allocvec(&graph_list).unwrap()
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
        "CSIRO" => {
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .from_reader(include_str!("../data/sealevel/csiro.tsv").as_bytes());
            let mut graph = Graph {
                name: &name,
                points: Vec::new(),
            };
            for result in rdr.deserialize() {
                let record: Row = result.map_err(|e| {
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
        "UHSLC" => {
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .from_reader(include_str!("../data/sealevel/uhslc.tsv").as_bytes());
            let mut graph = Graph {
                name: &name,
                points: Vec::new(),
            };
            for result in rdr.deserialize() {
                let record: Row = result.map_err(|e| {
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
        "Dev" => {
            let mut graph = Graph {
                name: &name,
                points: Vec::new(),
            };
            graph
                .points
                .push((NaiveDate::from_ymd_opt(0, 1, 1).unwrap(), 0.0f32));
            graph
                .points
                .push((NaiveDate::from_ymd_opt(0, 1, 2).unwrap(), 1.0f32));
            graph
                .points
                .push((NaiveDate::from_ymd_opt(0, 1, 3).unwrap(), 2.0f32));
            Ok(to_allocvec(&graph).map_err(|e| {
                error!("error encoding dataset {}: {}", name, e);
                error::ErrorInternalServerError("error encoding dataset")
            })?)
        }
        _ => Err(error::ErrorNotFound(format!("no graph with name {}", name))),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8999)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Listening on http://localhost:{}/ ...", args.port);
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
    .workers(1)
    .bind(format!("0.0.0.0:{}", args.port))?
    .run()
    .await
}
