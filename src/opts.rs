use std::path::Path;

use normpath::{BasePathBuf, PathExt};
use notify_debouncer_full::{notify::EventKind, DebouncedEvent};
use serde::{Deserialize, Serialize};

use crate::error::{WatchError, WatchResult};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum EventType {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "change")]
    Change,
    #[serde(rename = "delete")]
    Delete,
}

impl PartialEq<EventKind> for EventType {
    fn eq(&self, other: &EventKind) -> bool {
        match self {
            EventType::Create if other.is_create() => true,
            EventType::Change if other.is_modify() => true,
            EventType::Delete if other.is_remove() => true,
            _ => false,
        }
    }
}

impl EventType {
    pub fn from_notify_event(event: &DebouncedEvent) -> Option<Self> {
        match event.kind {
            EventKind::Create(_) => Some(EventType::Create),
            EventKind::Modify(_) => Some(EventType::Change),
            EventKind::Remove(_) => Some(EventType::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterOpts {
    #[serde(default = "RegisterOpts::default_cwd")]
    pub cwd: String,
    #[serde(default = "RegisterOpts::default_deb_changes")]
    pub debounce_changes: u64,
    #[serde(default = "RegisterOpts::default_watch_for")]
    pub watch_for: Vec<EventType>,
    #[serde(default = "RegisterOpts::default_patterns")]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub ignores: Vec<String>,
    pub uid: usize,
}

// for Deserialize onle
impl RegisterOpts {
    fn default_cwd() -> String {
        String::from(".")
    }

    const fn default_deb_changes() -> u64 {
        400
    }

    fn default_watch_for() -> Vec<EventType> {
        vec![EventType::Create, EventType::Change, EventType::Delete]
    }

    fn default_patterns() -> Vec<String> {
        vec![Self::default_cwd()]
    }
}

impl RegisterOpts {
    pub fn new(uid: usize) -> Self {
        Self {
            cwd: Self::default_cwd(),
            debounce_changes: Self::default_deb_changes(),
            watch_for: Self::default_watch_for(),
            patterns: Self::default_patterns(),
            ignores: Vec::new(),
            uid,
        }
    }

    pub fn patterns_to_paths(&self) -> WatchResult<(Vec<BasePathBuf>, Vec<WatchError>)> {
        Self::globs_to_paths(&self.validate_cwd(), &self.patterns)
    }

    pub fn ignores_to_paths(&self) -> WatchResult<(Vec<BasePathBuf>, Vec<WatchError>)> {
        Self::globs_to_paths(&self.validate_cwd(), &self.ignores)
    }

    fn validate_cwd(&self) -> String {
        Path::new(&self.cwd)
            .normalize()
            .expect("couldn't find cwd / current_dir")
            .into_path_buf()
            .display()
            .to_string()
    }

    fn globs_to_paths(
        cwd: &str,
        patterns: &[String],
    ) -> WatchResult<(Vec<BasePathBuf>, Vec<WatchError>)> {
        let mut paths = Vec::new();
        let mut invalid_patterns = Vec::new();
        for pat in patterns {
            let abs_pat = if Path::new(pat).is_absolute() {
                pat.to_owned()
            } else {
                cwd.to_owned() + "/" + pat
            };

            match glob::glob(&abs_pat) {
                Ok(entries) => {
                    let mut count = 0;
                    for entry in entries {
                        if let Ok(Ok(path)) = entry.map(|path| path.normalize()) {
                            paths.push(path);
                        }
                        count += 1;
                    }

                    // Experimental
                    if count == 0 {
                        invalid_patterns.push(WatchError::ExPatternError(pat.to_owned()))
                    }
                }
                Err(e) => invalid_patterns.push(WatchError::PatternError(pat.to_owned(), e)),
            }
        }
        Ok((paths, invalid_patterns))
    }
}
