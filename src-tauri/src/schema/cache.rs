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

use super::{message::MessagesSchema, segment::Field};

// simple cache for segments data based on file mod time
pub struct SchemaCache {
    messages_path: RwLock<PathBuf>,
    messages: RwLock<MessagesSchema>,
    message_mod_time: RwLock<SystemTime>,

    segments: RwLock<HashMap<String, Vec<Field>>>,
    segment_mod_time: RwLock<HashMap<String, SystemTime>>,
}

impl SchemaCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let messages = MessagesSchema::load_from_file(path).wrap_err_with(|| {
            format!(
                "Failed to load messages schema from {path}",
                path = path.display()
            )
        })?;

        let message_mod_time = std::fs::metadata(path)
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(SystemTime::now());

        Ok(Self {
            messages_path: RwLock::new(path.to_path_buf()),
            segments: RwLock::new(HashMap::new()),
            segment_mod_time: RwLock::new(HashMap::new()),
            messages: RwLock::new(messages),
            message_mod_time: RwLock::new(message_mod_time),
        })
    }

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

    fn path_for_segment(&self, segment: &str) -> Option<PathBuf> {
        let messages = self.messages.read().expect("Cannot read messages lock");
        let segment_path = messages.segments.get(segment).and_then(|path| {
            let path = PathBuf::from(path);
            if path.exists() {
                Some(path)
            } else {
                None
            }
        });

        segment_path
    }

    pub fn get_segment(&self, segment: &str) -> Result<Vec<Field>> {
        let path_for_segment = self
            .path_for_segment(segment)
            .wrap_err_with(|| format!("Failed to get path for segment {segment}"))?;
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

    pub fn get_messages(&self) -> Result<MessagesSchema> {
        let messages_path = self
            .messages_path
            .read()
            .expect("Cannot read messages path lock");
        let messages = self.messages.read().expect("Cannot read messages lock");
        let last_mod_time = *self
            .message_mod_time
            .read()
            .expect("Cannot read message mod time lock");
        let current_mod_time = std::fs::metadata(messages_path.as_path())
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(SystemTime::now());

        if current_mod_time != last_mod_time {
            log::debug!("Reloading messages schema from {messages_path:?}");
            if let Err(e) = MessagesSchema::load_from_file(messages_path.as_path()) {
                log::error!("Failed to reload messages schema: {e:#}");
            }
        }

        Ok(messages.clone())
    }
}
