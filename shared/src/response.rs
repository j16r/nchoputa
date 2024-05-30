use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq)]
pub struct GraphList {
    pub graphs: Vec<GraphSummary>,
}

#[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq)]
pub struct GraphIndex {
    pub graphs: HashMap<String, GraphSummary>,
}

// #[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq)]
#[derive(Clone, Debug)]
pub struct Graph {
    pub name: &'static str,
    pub description: &'static str,
    pub color: (u8, u8, u8),
    pub points: Points,
}

#[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq)]
pub struct GraphSummary {
    pub name: String,
    pub uri: String,
    pub description: String,
    pub color: (u8, u8, u8),
}

pub type Points = Vec<(NaiveDate, f32)>;

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
pub struct GraphData {
    pub name: String,
    pub color: (u8, u8, u8),
    pub points: Points,
}

impl GraphData {
    pub fn max_x(&self) -> NaiveDate {
        // self.points
        //     .iter()
        //     .map(|(x, _)| x)
        //     .fold(NaiveDate::MIN, |a, b| a.max(*b))
        // XXX: If we assume this and all graphs are left-right time series, this should always be
        // true?
        self.points
            .last()
            .and_then(|point| Some(point.0))
            .unwrap_or_default()
    }

    pub fn min_x(&self) -> NaiveDate {
        // self.points
        //     .iter()
        //     .map(|(x, _)| x)
        //     .fold(NaiveDate::MAX, |a, b| a.min(*b))
        // XXX: See above
        self.points
            .first()
            .and_then(|point| Some(point.0))
            .unwrap_or_default()
    }

    pub fn max_y(&self) -> f32 {
        self.points
            .iter()
            .map(|(_, y)| y)
            .fold(f32::MIN, |a, b| a.max(*b))
    }

    pub fn min_y(&self) -> f32 {
        self.points
            .iter()
            .map(|(_, y)| y)
            .fold(f32::MAX, |a, b| a.min(*b))
    }
}
