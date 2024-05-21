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

impl Graph {
    fn max_x(&self) -> NaiveDate {
        self.points
            .last()
            .and_then(|point| Some(point.0))
            .unwrap_or_default()
    }

    fn min_x(&self) -> NaiveDate {
        self.points
            .first()
            .and_then(|point| Some(point.0))
            .unwrap_or_default()
    }

    fn max_y(&self) -> f32 {
        self.points
            .first()
            .and_then(|point| Some(point.1))
            .unwrap_or_default()
    }

    fn min_y(&self) -> f32 {
        self.points
            .first()
            .and_then(|point| Some(point.1))
            .unwrap_or_default()
    }
}
