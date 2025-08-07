use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    text::{Line, Text},
    widgets::{Paragraph, Widget, Wrap},
};

#[derive(Debug)]
pub struct Fallback;

impl Widget for Fallback {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let area = area.centered_vertically(Constraint::Length(5));
        Paragraph::new(
            Text::from_iter(vec![
                Line::raw("Minimal resolution:"),
                Line::raw("w: 21, h: 16"),
                Line::raw("Provided:"),
                Line::raw(format!("w: {}, h: {}", area.width, area.height)),
            ])
            .centered(),
        )
        .wrap(Wrap { trim: true })
        .render(area, buf);
    }
}
