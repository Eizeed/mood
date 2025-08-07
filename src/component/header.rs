use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::component::Component;

#[derive(Debug)]
pub struct Header {
    pub playlist_name: String,
    pub area: Rect,
}

impl Header {
    pub fn new(playlist_name: String, area: Rect) -> Self {
        Header {
            playlist_name,
            area,
        }
    }
}

impl Component for Header {
    type Output = ();
    type Message = ();
    fn area(&self) -> Rect {
        self.area
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
    }

    fn update(&mut self, _message: Self::Message) -> Self::Output {}

    fn view(&self, buffer: &mut ratatui::prelude::Buffer) {
        Paragraph::new(Text::from(vec![Line::raw(self.playlist_name.as_str())]))
            .block(Block::new().borders(Borders::BOTTOM))
            .render(self.area, buffer);
    }
}
