use lofty::file::AudioFile;
use rusqlite::Connection;
use uuid::Uuid;

use crate::{component::search::Searchable, model};

#[derive(Debug, Clone)]
pub struct PlaylistMd {
    pub uuid: Uuid,
    pub name: String,
}

impl PlaylistMd {
    pub fn new(name: String) -> Self {
        let uuid = Uuid::new_v4();
        PlaylistMd { uuid, name }
    }

    pub fn save(self, conn: &Connection) {
        conn.execute(
            "INSERT INTO playlists (uuid, name) VALUES (?1, ?2);",
            [self.uuid.to_string(), self.name],
        )
        .unwrap();
    }

    pub fn insert_track(&self, track: model::Track, conn: &Connection) {
        let _ = conn.execute(
            r#"
                INSERT INTO playlist_tracks (playlist_uuid, track_uuid) VALUES (?1, ?2);
            "#,
            (self.uuid.to_string(), track.uuid.to_string()),
        );
    }

    pub fn get_all(conn: &Connection) -> Vec<PlaylistMd> {
        conn.prepare("SELECT uuid, name FROM playlists;")
            .unwrap()
            .query_map((), |r| {
                let uuid_str: Box<str> = r.get("uuid")?;
                let uuid = Uuid::parse_str(&uuid_str).unwrap();
                let name: String = r.get("name")?;

                Ok(PlaylistMd { uuid, name })
            })
            .unwrap()
            .map(|p| p.unwrap())
            .collect()
    }

    pub fn delete(self, conn: &Connection) {
        conn.execute(
            "DELETE FROM playlists WHERE uuid = ?1",
            [self.uuid.to_string()],
        )
        .unwrap();
    }
}

impl Searchable for PlaylistMd {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        self.name.as_str().into()
    }
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub uuid: Uuid,
    pub name: String,
    pub tracks: Vec<model::Track>,
}

impl Playlist {
    pub fn remove_track(mut self, track: model::Track, conn: &Connection) -> Self {
        conn.execute(
            "DELETE FROM playlist_tracks WHERE track_uuid = ?1 AND playlist_uuid = ?2",
            (track.uuid.to_string(), self.uuid.to_string()),
        )
        .unwrap();

        self.tracks.retain(|t| t.uuid != track.uuid);

        self
    }

    pub fn from_playlistmd(md: PlaylistMd, conn: &Connection) -> Playlist {
        let tracks = conn
            .prepare(
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
            .query([md.uuid.to_string()])
            .unwrap()
            .mapped(|row| {
                let track_uuid: Box<str> = row.get("track_uuid")?;
                let uuid = Uuid::parse_str(&track_uuid).unwrap();
                let path: String = row.get("track_path")?;
                let tagged = lofty::read_from_path(&path).unwrap();
                let duration = tagged.properties().duration();
                Ok(model::Track {
                    uuid,
                    path: path.into(),
                    duration,
                })
            })
            .filter_map(|t| t.ok())
            .collect();

        Playlist {
            uuid: md.uuid,
            name: md.name,
            tracks,
        }
    }
}

impl Searchable for Playlist {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        self.name.as_str().into()
    }
}
