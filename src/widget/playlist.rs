use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::{Line, Text},
    widgets::Widget,
};

use crate::model::{self, Track};

#[derive(Debug)]
pub struct Playlist {
    pub list: Vec<model::Playlist>,

    pub selected_track: Option<Track>,

    pub cursor: u16,
    pub show_cursor: bool,

    pub y_offset: u16,

    pub area: Rect,
}

impl Playlist {
    pub fn new(playlists: Vec<model::Playlist>, area: Rect) -> Self {
        Playlist {
            list: playlists,
            selected_track: None,
            cursor: 0,
            show_cursor: true,
            y_offset: 0,
            area,
        }
    }

    pub fn resize(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn get_under_cursor(&self) -> model::Playlist {
        let index = (self.cursor + self.y_offset) as usize;
        self.list[index].clone()
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
        let total = self.list.len() as u16;

        if self.cursor + (count as u16) < self.area.height
            && self.y_offset + self.cursor + (count as u16) < total
        {
            self.cursor += count as u16;
        } else if self.y_offset + self.area.height - 1 < total - 1 {
            self.y_offset += 1;
        }
    }
}

impl Widget for &Playlist {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let w = area.width;
        let y = self.cursor + area.y;

        for x in 0..w {
            buf.cell_mut((x, y)).unwrap().set_fg(Color::Green);
        }

        let list = Text::from_iter(
            self.list
                .iter()
                .skip(self.y_offset as usize)
                .map(|t| Line::raw(&t.name)),
        );

        list.render(area, buf);
    }
}
