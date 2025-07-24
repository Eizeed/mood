use std::{path::PathBuf, time::Duration};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Track {
    pub uuid: Uuid,
    pub duration: Duration,
    pub path: PathBuf,
}
