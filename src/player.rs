use ratatui::widgets::{List, ListItem, Widget};

use crate::cursor::Cursor;

pub struct Player {
    pub tracks: Vec<String>,
    pub current: Option<String>,
    pub cursor: Cursor,
    pub is_paused: bool,
}

impl Player {
    pub fn new(path: &str) -> Self {
        let tracks = std::fs::read_dir(path).unwrap();

        let names: Vec<String> = tracks
            .map(|e| {
                let entry = e.unwrap();
                entry.path().to_str().unwrap().to_string()
            })
            .collect();

        Player {
            tracks: names,
            current: None,
            cursor: Cursor::new(),
            is_paused: false,
        }
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
