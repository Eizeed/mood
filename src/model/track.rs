use std::{path::PathBuf, time::Duration};

use uuid::Uuid;

use crate::model::playlist::DbTrack;

#[derive(Debug, Clone)]
pub struct Track {
    pub index: usize,
    pub uuid: Uuid,
    pub duration: Duration,
    pub path: PathBuf,
}

impl Track {
    pub fn from_db_tracks(tracks: &[Track], mut db_tracks: Vec<DbTrack>) -> Vec<Track> {
        let mut converted = Vec::with_capacity(db_tracks.capacity());

        while let Some(db_track) = db_tracks.pop() {
            let track = tracks
                .iter()
                .find(|t| t.uuid == db_track.track_uuid)
                .unwrap();
            converted.push(track.clone());
        }
        converted
    }
}
