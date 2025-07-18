use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Widget,
};

use crate::model::{self, Track};

#[derive(Debug)]
pub struct Tracklist {
    pub base: Vec<Track>,

    pub list: Vec<Track>,
    pub auto_queue: VecDeque<Track>,
    pub manual_queue: VecDeque<Track>,
    pub history: Vec<Track>,

    pub current_track: Option<Track>,

    pub selected_playlist: Option<(model::Playlist, Vec<Track>)>,

    pub cursor: u16,
    pub show_cursor: bool,

    pub y_offset: u16,

    pub area: Rect,
}

impl Tracklist {
    pub fn new(paths: Vec<Track>, area: Rect) -> Self {
        Tracklist {
            base: paths.clone(),
            list: paths,
            auto_queue: VecDeque::new(),
            manual_queue: VecDeque::new(),
            history: Vec::new(),
            current_track: None,
            selected_playlist: None,
            cursor: 0,
            show_cursor: true,
            y_offset: 0,
            area,
        }
    }

    pub fn get_under_cursor(&self) -> Track {
        let index = (self.cursor + self.y_offset) as usize;

        match &self.selected_playlist {
            Some((_, tracks)) => {
                assert!(index < tracks.len(), "Index of cursor is out of bounds");

                tracks[index].clone()
            },
             None => {
                assert!(index < self.base.len(), "Index of cursor is out of bounds");

                self.base[index].clone()
             }
        }
    }

    pub fn cursor_up(&mut self, count: u16) {
        if self.cursor < count {
            let rest = count - self.cursor;
            self.y_offset = self.y_offset.saturating_sub(rest);
        } else {
            self.cursor -= count;
        }
    }

    pub fn cursor_down(&mut self, count: u16) {
        let total = self
            .selected_playlist
            .as_ref()
            .map(|(_, tracks)| tracks.len() as u16)
            .unwrap_or(self.base.len() as u16);

        if self.cursor + (count as u16) < self.area.height
            && self.y_offset + self.cursor + (count as u16) < total
        {
            self.cursor += count as u16;
        } else if self.y_offset + self.area.height - 1 < total - 1 {
            self.y_offset += 1;
        }
    }

    pub fn resize(&mut self, area: Rect) {
        self.area = area;
    }
}

impl Widget for &Tracklist {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let w = area.width;
        let y = self.cursor + area.y;
        for x in 0..w {
            buf.cell_mut((x, y)).unwrap().set_fg(Color::Green);
        }

        let current = self.current_track.as_ref();
        let list: &[Track] = self
            .selected_playlist
            .as_ref()
            .map(|(_, tracks)| &**tracks) // LOL
            .unwrap_or(&*self.base);

        let list = match current {
            Some(current) => Text::from_iter(
                list.into_iter()
                    .skip(self.y_offset as usize)
                    .enumerate()
                    .map(|(i, t)| {
                        let path = t.path.to_string_lossy();
                        let name = path.split("/").last().unwrap().to_string();

                        // NOTE: Idk what this is doing (i wrote it)
                        // spend some time in future to understand
                        let line = if current.path.to_string_lossy().contains(&name)
                            && (self.y_offset..self.y_offset + self.area.height)
                                .contains(&(i as u16))
                        {
                            let color = if self.base[(self.cursor + self.y_offset) as usize].uuid
                                == current.uuid
                            {
                                Color::Yellow
                            } else {
                                Color::Blue
                            };

                            Line::raw(name).fg(color)
                        } else {
                            Line::raw(name)
                        };

                        line
                    }),
            ),
            None => Text::from_iter(list.into_iter().skip(self.y_offset as usize).map(|t| {
                Line::raw(
                    t.path
                        .to_string_lossy()
                        .split("/")
                        .last()
                        .unwrap()
                        .to_string(),
                )
            })),
        };

        list.render(area, buf);
    }
}
