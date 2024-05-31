use actix_files as fs;
use actix_web::{
    error, get, http::header, middleware, web, App, HttpResponse, HttpServer, Responder, Result,
};
use chrono::NaiveDate;
use clap::Parser;
use postcard::to_allocvec;

use shared::response::{GraphData, GraphList, GraphSummary};
use tracing::{error, info};

mod graphs;

#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[get("/api/graphs")]
async fn list_graphs() -> impl Responder {
    let list: Vec<GraphSummary> = graphs::INDEX
        .read()
        .unwrap()
        .iter()
        .map(|(_, graph)| GraphSummary {
            name: graph.name.to_string(),
            uri: format!("/api/graphs/{}", graph.name),
            description: graph.description.to_string(),
            color: graph.color,
        })
        .collect();
    to_allocvec(&GraphList { graphs: list }).map_err(|e| {
        error!("error encoding graph index: {}", e);
        error::ErrorInternalServerError("error encoding graph index")
    })
}

//
// EAFDCF
// 8E8358
//
#[get("/api/graphs/{name}")]
async fn show_graph(name: web::Path<String>) -> Result<impl Responder> {
    let graph = match graphs::INDEX.read().unwrap().get(name.as_str()) {
        Some(graph) => GraphData {
            name: graph.name.to_string(),
            color: graph.color,
            points: graph.points.clone(),
        },
        None if name.as_str() == "Dev" => {
            let points = vec![
                (NaiveDate::from_ymd_opt(0, 1, 1).unwrap(), 0.0f32),
                (NaiveDate::from_ymd_opt(0, 1, 2).unwrap(), 1.0f32),
                (NaiveDate::from_ymd_opt(0, 1, 3).unwrap(), 2.0f32),
            ];
            GraphData {
                name: name.to_string(),
                color: (0xEA, 0xFD, 0xCF),
                points,
            }
        }
        _ => return Err(error::ErrorNotFound(format!("no graph with name {}", name))),
    };

    to_allocvec(&graph).map_err(|e| {
        error!("error encoding dataset {}: {}", name, e);
        error::ErrorInternalServerError("error encoding dataset")
    })
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
