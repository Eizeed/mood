use std::path::PathBuf;

use rusqlite::{Connection, fallible_iterator::FallibleIterator};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Playlist {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DbTrack {
    pub track_uuid: Uuid,
    pub path: PathBuf,
}

impl DbTrack {
    pub fn get_by_playlist_uuid(conn: &Connection, playlist_uuid: Uuid) -> Vec<Self> {
        conn.prepare(
            r#"
                SELECT 
                    tracks.uuid AS track_uuid,
                    tracks.path AS track_path
                FROM playlists
                LEFT JOIN track_playlist ON playlists.uuid = track_playlist.playlist_uuid
                LEFT JOIN tracks ON tracks.uuid = track_playlist.track_uuid
                WHERE playlists.uuid = ?1;
            "#,
        )
        .unwrap()
        .query([playlist_uuid.to_string()])
        .unwrap()
        .map(|row| {
            let track_uuid: Box<str> = row.get("track_uuid")?;
            let uuid = Uuid::parse_str(&track_uuid).unwrap();
            let path: String = row.get("track_path")?;
            Ok(DbTrack {
                track_uuid: uuid,
                path: path.into(),
            })
        })
        .unwrap()
        .collect()
    }
}
