use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct GraphList {
    pub graphs: HashMap<String, String>,
}

pub type Points = Vec<(NaiveDate, f32)>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Graph<'a> {
    pub name: &'a str,
    pub points: Points,
}
