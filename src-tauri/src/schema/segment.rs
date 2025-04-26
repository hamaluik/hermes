use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Date,
    DateTime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Field {
    pub field: u8,
    pub name: String,
    pub component: Option<u8>,
    pub group: Option<String>,
    pub trigger_filter: Option<String>,
    pub minlength: Option<u16>,
    pub maxlength: Option<u16>,
    pub placeholder: Option<String>,
    pub required: Option<bool>,
    pub datatype: Option<DataType>,
    pub pattern: Option<String>,
    pub note: Option<String>,
    pub values: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Fields {
    fields: Vec<Field>,
}

impl Field {
    // not async because I'm not smart enough to figure it out without wrapping
    // the entire schema cache in a single mutex which defeats the entire purpose
    // of async to begin with
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read file {:?}", path.display()))?;
        let fields: Fields = toml::from_str(&contents)
            .wrap_err_with(|| format!("Failed to parse file {:?}", path.display()))?;
        Ok(fields.fields)
    }
}
