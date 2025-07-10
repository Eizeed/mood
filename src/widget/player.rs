use std::{collections::VecDeque, time::Duration};

use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{List, ListItem, Widget},
};

use super::Cursor;

pub struct Player {
    tracks: Vec<String>,
    index: Option<usize>,
    current: Option<CurrentTrack>,
    queue: VecDeque<Track>,
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
            index: None,
            queue: VecDeque::new(),
            cursor: Cursor::new(),
            is_paused: false,
        }
    }

    pub fn queue_mut(&mut self) -> &mut VecDeque<Track> {
        &mut self.queue
    }

    pub fn tracks(&self) -> &[String] {
        &self.tracks
    }

    pub fn tracks_len(&self) -> usize {
        self.tracks.len()
    }

    pub fn track_under_cursor(&self) -> String {
        self.tracks[self.cursor.y as usize].clone()
    }

    pub fn get_current_path(&self) -> Option<&str> {
        self.current.as_ref().map(|s| s.path.as_str())
    }

    pub fn get_current_duration(&self) -> Option<Duration> {
        self.current.as_ref().map(|s| s.duration)
    }

    pub fn get_current_index(&self) -> Option<usize> {
        self.current.as_ref().map(|c| c.index)
    }

    pub fn get_current(&self) -> Option<&CurrentTrack> {
        self.current.as_ref()
    }

    pub fn set_current<T: Into<usize>>(&mut self, str: String, duration: Duration, index: T) {
        self.current = Some(CurrentTrack {
            path: str.into(),
            duration,
            index: index.into(),
        })
    }

    pub fn unset_current(&mut self) {
        self.current = None;
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

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = Some(index);
    }
}

impl Widget for &mut Player {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let current = self.current.as_ref();
        let list = match current {
            Some(current) => Text::from(
                self.tracks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let name = t.split("/").last().unwrap();

                        let line = if current.path.contains(name) && current.index == i {
                            Line::raw(name).fg(Color::Blue)
                        } else {
                            Line::raw(name)
                        };

                        line
                    })
                    .collect::<Vec<Line>>(),
            ),
            None => Text::from(
                self.tracks
                    .iter()
                    .map(|t| {
                        let name = t.split("/").last().unwrap();
                        let line = Line::raw(name);

                        line
                    })
                    .collect::<Vec<Line>>(),
            ),
        };

        list.render(area, buf);

        self.cursor.render(area, buf);
    }
}

#[derive(Debug)]
pub struct CurrentTrack {
    pub path: String,
    pub duration: Duration,
    pub index: usize,
}

#[derive(Debug)]
pub struct Track {
    pub path: String,
    pub index: usize,
}
