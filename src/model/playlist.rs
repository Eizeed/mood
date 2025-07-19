use std::path::PathBuf;

use rusqlite::{Connection, fallible_iterator::FallibleIterator};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Playlist {
    pub uuid: Uuid,
    pub name: String,
}

impl Playlist {
    pub fn get_all(conn: &Connection) -> Vec<Playlist> {
        let mut stmt = conn.prepare("SELECT uuid, name FROM playlists;").unwrap();

        stmt.query_map((), |row| {
            let uuid: Box<str> = row.get("uuid")?;
            let uuid = Uuid::parse_str(&uuid).unwrap();

            let name: String = row.get("name")?;

            Ok(Playlist { uuid, name })
        })
        .unwrap()
        .into_iter()
        .map(|r| r.unwrap())
        .collect()
    }

    pub fn delete(self, conn: &mut Connection) {
        let tx = conn.transaction().unwrap();
        tx.execute(
            r#"
                DELETE FROM playlist_tracks WHERE playlist_uuid = ?1
            "#,
            [self.uuid.to_string()],
        )
        .unwrap();
        tx.execute(
            r#"
                DELETE FROM playlists WHERE uuid = ?1
            "#,
            [self.uuid.to_string()],
        )
        .unwrap();

        tx.commit().unwrap();
    }
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
                LEFT JOIN playlist_tracks ON playlists.uuid = playlist_tracks.playlist_uuid
                LEFT JOIN tracks ON tracks.uuid = playlist_tracks.track_uuid
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
