use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

pub type SegmentPaths = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SegmentMetadata {
    pub name: String,
    pub required: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MessagesSchema {
    pub segments: SegmentPaths,
    pub message: HashMap<String, Vec<SegmentMetadata>>,
}

impl MessagesSchema {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read file {:?}", path.display()))?;
        let schema: MessagesSchema = toml::from_str(&contents)
            .wrap_err_with(|| format!("Failed to parse file {:?}", path.display()))?;
        Ok(schema)
    }
}
