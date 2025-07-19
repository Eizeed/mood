use std::{path::PathBuf, time::Duration};

use rusqlite::Connection;
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

    pub fn insert_into_playlist(self, playlist_uuid: Uuid, conn: &Connection) {
        conn.execute(
            r#"
                INSERT INTO playlist_tracks
                (track_uuid, playlist_uuid)
                VALUES
                (?1, ?2);
            "#,
            [self.uuid.to_string(), playlist_uuid.to_string()],
        )
        .unwrap();
    }

    pub fn delete_from_playliost(self, playlist_uuid: Uuid, conn: &Connection) {
        conn.execute(
            r#"
                DELETE FROM playlist_tracks
                WHERE track_uuid = ?1 AND playlist_uuid = ?2;
            "#,
            [self.uuid.to_string(), playlist_uuid.to_string()],
        )
        .unwrap();
    }
}
