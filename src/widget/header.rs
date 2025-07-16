use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct Header {
    pub playlist_name: String,
    pub area: Rect,
}

impl Header {
    pub const HEIGHT: u16 = 2;

    pub fn new(playlist_name: String, area: Rect) -> Self {
        Header {
            playlist_name,
            area,
        }
    }

    pub fn resize(&mut self, area: Rect) {
        self.area = area;
    }
}

impl Widget for &Header {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(Text::from(vec![Line::raw(self.playlist_name.as_str())]))
            .block(Block::new().borders(Borders::BOTTOM))
            .render(area, buf);
    }
}
