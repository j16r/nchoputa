use std::sync::RwLock;
use std::{collections::HashMap, fs::File, io::Result};

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use serde::Deserialize;

use shared::response::{Graph, Points};

static GRAPHS: Lazy<RwLock<Vec<Graph>>> = Lazy::new(|| {
    RwLock::new(vec![
        Graph{
            name: "CSIRO",
            description: "Change in sea level in millimeters compared to the 1993-2008 average from the sea level group of CSIRO (Commonwealth Scientific and Industrial Research Organisation), Australia's national science agency. It is based on the paper Church, J. A., & White, N. J. (2011). Sea-Level Rise from the Late 19th to the Early 21st Century. Surveys in Geophysics, 32(4), 585Ð602. https://doi.org/10.1007/s10712-011-9119-1.",
            points: points_from_tsv("sealevel/csiro").unwrap(),
            color: (0xB1, 0xF8, 0xF2),
        },
        Graph{
            name: "UHSLC",
            description: "Change in sea level in millimeters compared to the 1993-2008 average from the University of Hawaii Sea Level Center (http://uhslc.soest.hawaii.edu/data/?fd). It is based on a weighted average of 373 global tide gauge records collected by the U.S. National Ocean Service, UHSLC, and partner agencies worldwide.",
            points: points_from_tsv("sealevel/uhslc").unwrap(),
            color: (0xBC, 0xD3, 0x9C),
        },
    ])
});

pub static INDEX: Lazy<RwLock<HashMap<String, Graph>>> = Lazy::new(|| {
    RwLock::new(
        GRAPHS
            .read()
            .unwrap()
            .iter()
            .fold(HashMap::new(), |mut acc, graph| {
                acc.insert(graph.name.to_string(), graph.clone());
                acc
            }),
    )
});

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Row {
    Date: NaiveDate,
    Value: f32,
}

fn points_from_tsv(path: &str) -> Result<Points> {
    let file = File::open(format!("data/{}.tsv", path))?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(file);
    let mut points = Vec::new();
    for result in rdr.deserialize() {
        let record: Row = result?;
        points.push((record.Date, record.Value));
    }
    Ok(points)
}
