use std::{path::PathBuf, time::Duration};

pub struct CurrentTrack {
    pub path: PathBuf,
    pub total_duration: Duration,
}

impl CurrentTrack {
    pub fn new(path: PathBuf, total_duration: Duration) -> Self {
        CurrentTrack {
            path,
            total_duration,
        }
    }

    pub fn name(&self) -> String {
        self.path.file_stem().unwrap().to_string_lossy().to_string()
    }
}
