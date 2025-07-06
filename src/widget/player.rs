use ratatui::widgets::{List, ListItem, Widget};

use super::Cursor;

pub struct Player {
    tracks: Vec<String>,
    current: Option<String>,
    cursor: Cursor,
    is_paused: bool,
}

impl Player {
    pub fn new(path: &str) -> Self {
        let tracks = std::fs::read_dir(path).unwrap();

        // Do better
        let names: Vec<String> = tracks
            .map(|e| {
                let entry = e.unwrap();
                entry.path().to_str().unwrap().to_string()
            })
            .filter(|n| n.ends_with(".mp3"))
            .collect();

        Player {
            tracks: names,
            current: None,
            cursor: Cursor::new(),
            is_paused: false,
        }
    }

    pub fn tracks_len(&self) -> usize {
        self.tracks.len()
    }

    pub fn track_under_cursor(&self) -> String {
        self.tracks[self.cursor.y as usize].clone()
    }

    pub fn get_current(&self) -> Option<&str> {
        self.current.as_ref().map(|s| s.as_str())
    }

    pub fn set_current(&mut self, str: String) {
        self.current = Some(str)
    }

    pub fn cursor(&self) -> u16 {
        self.cursor.y
    }

    pub fn set_cursor(&mut self, y: u16) {
        self.cursor.y = y;
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, is_paused: bool) {
        self.is_paused = is_paused;
    }
}

impl Widget for &mut Player {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let list = List::new(
            self.tracks
                .iter()
                .map(|t| ListItem::new(t.split("/").last().unwrap())),
        );

        list.render(area, buf);

        self.cursor.render(area, buf);
    }
}
