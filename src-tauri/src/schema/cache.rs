//! Schema caching with automatic reload on file modification.
//!
//! This module provides a thread-safe cache for HL7 schema data loaded from TOML files.
//! The cache tracks file modification times and automatically reloads schemas when
//! files change, enabling hot-reload during development.
//!
//! # Why File-Based Schemas?
//! HL7 schemas are stored in TOML files rather than being hard-coded to allow:
//! * Easy customization for different HL7 versions
//! * Addition of custom segments without recompiling
//! * Documentation of field purposes directly in the schema files
//!
//! # Cache Strategy
//! The cache uses RwLock for concurrent reads with exclusive writes. Each schema type
//! (messages, segments) has its own RwLock to minimise contention. File modification
//! times are compared before each access to detect changes.

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

/// Thread-safe cache for HL7 schema data with hot-reload capability.
///
/// Stores both message-level schemas (which messages exist, what segments they contain)
/// and segment-level schemas (what fields each segment has). Automatically reloads
/// from disk when source files are modified.
pub struct SchemaCache {
    /// Path to the messages schema TOML file
    messages_path: RwLock<PathBuf>,
    /// Cached messages schema
    messages: RwLock<MessagesSchema>,
    /// Last modification time of the messages schema file
    message_mod_time: RwLock<SystemTime>,

    /// Cached segment schemas (keyed by segment name)
    segments: RwLock<HashMap<String, Vec<Field>>>,
    /// Last modification times of segment schema files (keyed by segment name)
    segment_mod_time: RwLock<HashMap<String, SystemTime>>,
}

impl SchemaCache {
    /// Create a new schema cache by loading the messages schema file.
    ///
    /// Segment schemas are loaded lazily on first access. The path is relative
    /// to the "data" directory in the application bundle.
    ///
    /// # Arguments
    /// * `path` - Relative path to the messages schema TOML file (e.g., "messages.toml")
    ///
    /// # Returns
    /// * `Ok(SchemaCache)` - Initialized cache with messages schema loaded
    /// * `Err` - Failed to load or parse messages schema file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let path = PathBuf::from("data").join(path);
        let messages = MessagesSchema::load_from_file(&path).wrap_err_with(|| {
            format!(
                "Failed to load messages schema from {path}",
                path = path.display()
            )
        })?;

        let message_mod_time = std::fs::metadata(&path)
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(SystemTime::now());

        Ok(Self {
            messages_path: RwLock::new(path),
            segments: RwLock::new(HashMap::new()),
            segment_mod_time: RwLock::new(HashMap::new()),
            messages: RwLock::new(messages),
            message_mod_time: RwLock::new(message_mod_time),
        })
    }

    /// Load a segment schema from disk and store it in the cache.
    ///
    /// This method acquires write locks on both the segments map and the modification
    /// time map, so it blocks concurrent reads briefly during loading.
    ///
    /// # Arguments
    /// * `segment` - Segment name (e.g., "PID", "MSH")
    /// * `path` - File path to the segment schema TOML file
    ///
    /// # Returns
    /// * `Ok(())` - Segment loaded and cached successfully
    /// * `Err` - Failed to load or parse segment schema file
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

    /// Resolve the file path for a segment schema.
    ///
    /// Looks up the segment path in the messages schema and resolves it relative
    /// to the "data" directory. Returns None if the segment is not defined in the
    /// messages schema or if the file doesn't exist.
    ///
    /// # Arguments
    /// * `segment` - Segment name to look up
    ///
    /// # Returns
    /// * `Some(PathBuf)` - Absolute path to the segment schema file
    /// * `None` - Segment not found in messages schema or file doesn't exist
    fn path_for_segment(&self, segment: &str) -> Option<PathBuf> {
        let messages = self.messages.read().expect("Cannot read messages lock");
        let segment_path = messages.segments.get(segment).and_then(|path| {
            let path = PathBuf::from("data").join(path);
            if path.exists() {
                Some(path)
            } else {
                None
            }
        });

        segment_path
    }

    /// Get a segment schema, loading or reloading from disk as needed.
    ///
    /// This method implements the core caching logic:
    /// 1. Check if segment is in cache
    /// 2. If not, load from disk and cache it
    /// 3. If yes, check if file has been modified
    /// 4. If modified, reload from disk
    /// 5. Return the cached (possibly reloaded) schema
    ///
    /// # Why Check Modification Time Every Access?
    /// This enables hot-reload during development. When schema files are edited,
    /// the changes are automatically picked up without restarting the application.
    /// The filesystem metadata check is relatively cheap compared to parsing TOML.
    ///
    /// # Thread Safety
    /// Uses read locks for checking cache state and write locks only when loading.
    /// This allows concurrent reads while ensuring exclusive access during loads.
    ///
    /// # Arguments
    /// * `segment` - Segment name to retrieve
    ///
    /// # Returns
    /// * `Ok(Vec<Field>)` - Field definitions for the segment
    /// * `Err` - Segment not found in messages schema or failed to load
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
                    segments
                        .get(segment)
                        .cloned()
                        .wrap_err_with(|| format!("Failed to load segment {segment} from cache"))
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

    /// Get the messages schema, reloading from disk if modified.
    ///
    /// Similar to `get_segment`, this method checks the file modification time
    /// and reloads the schema if the file has changed since it was last loaded.
    ///
    /// # Returns
    /// * `Ok(MessagesSchema)` - The messages schema (possibly reloaded)
    /// * `Err` - Should not happen as the schema is loaded successfully in `new()`
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
