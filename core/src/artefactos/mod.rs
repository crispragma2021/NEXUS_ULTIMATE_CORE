use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Artifact {
    pub task_id: String,
    pub task_list: Vec<String>,
    pub implementation_plan: String,
    pub screenshots: Vec<PathBuf>,
    pub audit_report: Option<String>,
}

impl Artifact {
    pub fn new(id: &str) -> Self {
        Self {
            task_id: id.to_string(),
            task_list: Vec::new(),
            implementation_plan: String::new(),
            screenshots: Vec::new(),
            audit_report: None,
        }
    }
}
