use std::{borrow::Cow, path::PathBuf, time::Duration};

use uuid::Uuid;

use crate::component::search::Searchable;

#[derive(Debug, Clone)]
pub struct Track {
    pub uuid: Uuid,
    pub duration: Duration,
    pub path: PathBuf,
}

impl Searchable for Track {
    fn name(&self) -> Cow<'_, str> {
        self.path
            .as_path()
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or("None".into())
    }
}
