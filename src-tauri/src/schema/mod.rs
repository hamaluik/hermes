use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

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

// simple cache for segments data based on file mod time
#[derive(Default)]
pub struct SchemaCache {
    segments: RwLock<HashMap<String, Vec<Field>>>,
    segment_mod_time: RwLock<HashMap<String, SystemTime>>,
}

impl SchemaCache {
    fn load_segment<P: AsRef<Path>>(&self, segment: &str, path: P) -> Result<()> {
        let path = path.as_ref();

        let fields = Field::load_from_file(path).wrap_err_with(|| {
            format!("Failed to load fields from {path}", path = path.display())
        })?;
        let mut segments = self.segments.write().expect("Cannot write segments lock");
        segments.insert(segment.to_string(), fields);
        drop(segments);

        let mod_time = std::fs::metadata(path)
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(SystemTime::now());
        let mut segment_mod_time = self
            .segment_mod_time
            .write()
            .expect("Can write segment mod times lock");
        segment_mod_time.insert(segment.to_string(), mod_time);
        drop(segment_mod_time);

        Ok(())
    }

    fn path_for_segment(segment: &str) -> PathBuf {
        PathBuf::from(format!("{segment}.toml"))
    }

    pub fn get_segment(&self, segment: &str) -> Result<Vec<Field>> {
        let path_for_segment = Self::path_for_segment(segment);
        let segments = self.segments.read().expect("Cannot read segments lock");
        let has_segment = segments.contains_key(segment);
        drop(segments);

        if !has_segment {
            return self
                .load_segment(segment, &path_for_segment)
                .wrap_err_with(|| {
                    format!("Failed to load segment {segment} from {path_for_segment:?}")
                })
                .and_then(|()| {
                    let segments = self.segments.read().expect("Cannot read segments lock");
                    Ok(segments
                        .get(segment)
                        .cloned()
                        .wrap_err_with(|| format!("Failed to load segment {segment} from cache"))?)
                });
        }

        let segment_mod_time = self
            .segment_mod_time
            .read()
            .expect("Cannot read segment mod times lock");
        let last_mod_time = segment_mod_time
            .get(segment)
            .cloned()
            .unwrap_or(SystemTime::now());
        drop(segment_mod_time);

        let current_mod_time = std::fs::metadata(&path_for_segment)
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(SystemTime::now());
        if current_mod_time != last_mod_time {
            log::debug!("Reloading segment {segment} from {path_for_segment:?}");
            if let Err(e) = self.load_segment(segment, &path_for_segment) {
                log::error!("Failed to reload segment {segment}: {e:#}");
            }
        }

        let segments = self.segments.read().expect("Cannot read segments lock");
        segments
            .get(segment)
            .cloned()
            .wrap_err_with(|| format!("Failed to load segment {segment} from cache"))
    }
}
