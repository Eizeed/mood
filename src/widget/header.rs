use ratatui::{text::Line, widgets::Widget};

pub struct Header {
    pub msg: String,
}

impl Widget for &Header {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Line::raw(self.msg.as_str()).render(area, buf)
    }
}
