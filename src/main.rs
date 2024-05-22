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

//
// EAFDCF
// 8E8358
//
#[get("/api/graphs/{name}")]
async fn show_graph(name: web::Path<String>) -> Result<impl Responder> {
    match name.as_str() {
        "CSIRO" => {
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .from_reader(include_str!("../data/sealevel/csiro.tsv").as_bytes());
            let mut graph = Graph {
                name: name.to_string(),
                description: "Change in sea level in millimeters compared to the 1993-2008 average from the sea level group of CSIRO (Commonwealth Scientific and Industrial Research Organisation), Australia's national science agency. It is based on the paper Church, J. A., & White, N. J. (2011). Sea-Level Rise from the Late 19th to the Early 21st Century. Surveys in Geophysics, 32(4), 585Ð602. https://doi.org/10.1007/s10712-011-9119-1. ".to_string(),
                points: Vec::new(),
                color: (0xB1, 0xF8, 0xF2),
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
                name: name.to_string(),
                description: "Change in sea level in millimeters compared to the 1993-2008 average from the University of Hawaii Sea Level Center (http://uhslc.soest.hawaii.edu/data/?fd). It is based on a weighted average of 373 global tide gauge records collected by the U.S. National Ocean Service, UHSLC, and partner agencies worldwide.".to_string(),
                points: Vec::new(),
                color: (0xBC, 0xD3, 0x9C),
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
                name: name.to_string(),
                description: "Graph for development, 3 simple points.".to_string(),
                points: Vec::new(),
                color: (0xFF, 0xFC, 0x99),
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
