use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct GraphList {
    pub graphs: HashMap<String, String>,
}

pub type Points = Vec<(NaiveDate, f32)>;

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
pub struct Graph {
    pub name: String,
    pub points: Points,
    pub color: (u8, u8, u8),
}
