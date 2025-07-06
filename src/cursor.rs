use ratatui::{style::Color, widgets::Widget};

pub struct Cursor {
    pub y: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor { y: 0 }
    }
}

impl Widget for &mut Cursor {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let w = area.width;
        let y = self.y + area.y;
        for x in 0..w {
            let cell = buf.cell_mut((x, y)).unwrap();
            cell.set_bg(Color::Yellow);
        }
    }
}
