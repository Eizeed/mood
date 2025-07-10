use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::Widget,
};

#[derive(Debug)]
pub struct ControlBar {
    pub name: String,
    // duration
    // pos
    pub repeat: bool,
    pub random: bool,
    pub progress: Option<f32>,
}

impl ControlBar {
    pub fn new() -> Self {
        ControlBar {
            name: "".to_string(),
            repeat: false,
            random: false,
            progress: None,
        }
    }
}

impl Widget for &ControlBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        debug_assert!(area.height == 3);

        let [name_area, progress_area, _button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(area);

        {
            Line::raw(&self.name).centered().render(name_area, buf);
        }

        let [_, progress_area, _] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Fill(8),
                Constraint::Fill(1),
            ],
        )
        .areas(progress_area);

        {
            let width = progress_area.width;
            let progress = match self.progress {
                Some(progress) => progress * 100.0,
                None => 0.0,
            };

            let one_cell_rat = 100.0 / width as f32;

            let till = (progress / one_cell_rat).round() as u16;
            for x in 0..width {
                let cell = buf
                    .cell_mut((progress_area.x + x, progress_area.y))
                    .unwrap();
                if x < till {
                    cell.set_char('#');
                } else {
                    cell.set_char('_');
                }
            }
        };
    }
}
