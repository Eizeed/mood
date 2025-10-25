use crossbeam_channel::Sender;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Paragraph},
};

use super::{Component, Widget, WidgetRef};
use crate::{
    components::utils::VerticalScroll,
    config::KeyConfig,
    event::{Command, EventState},
    models::Track,
};
use color_eyre::Result;

pub struct TracklistComponent {
    library: Vec<Track>,
    current_track: Option<usize>,
    scroll: VerticalScroll,
    key_config: KeyConfig,
    command_tx: Sender<Command>,
}

impl TracklistComponent {
    pub fn new(lib: Vec<Track>, key_config: KeyConfig, command_tx: Sender<Command>) -> Self {
        Self {
            library: lib,
            current_track: None,
            scroll: VerticalScroll::new(),
            key_config,
            command_tx,
        }
    }

    fn next_col(&self) {
        self.scroll.move_down(self.library.len());
    }

    fn prev_col(&self) {
        self.scroll.move_up();
    }

    fn play_selected(&mut self) -> Result<()> {
        let index = self.scroll.pos();
        let path = self.library.get(index).unwrap().path.as_path();
        self.current_track = Some(index);
        self.command_tx.send(Command::Play(path.to_path_buf()))?;

        Ok(())
    }
}

impl WidgetRef for TracklistComponent {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = {
            let border = Block::bordered();
            let a = border.inner(area);
            border.render(area, buf);
            a
        };

        self.scroll.update(area.height as usize, self.library.len());

        let tracks = self
            .library
            .iter()
            .skip(self.scroll.y_offset.get())
            .take(area.height as usize)
            .map(|t| {
                t.path
                    .to_path_buf()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .split('[')
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();

        Paragraph::new(tracks.join("\n")).render(area, buf);

        if !self.library.is_empty() {
            let selection = self.scroll.pos.get() - self.scroll.y_offset.get();
            for i in 0 + area.x..=area.width {
                buf.cell_mut((i, selection as u16 + area.y))
                    .map(|c| c.set_bg(Color::Blue));
            }
        }
    }
}

impl Component for TracklistComponent {
    fn event(&mut self, key: crate::event::Key) -> Result<EventState> {
        if key == self.key_config.scroll_up {
            self.prev_col();
            Ok(EventState::Consumed)
        } else if key == self.key_config.scroll_down {
            self.next_col();
            Ok(EventState::Consumed)
        } else if key == self.key_config.play_audio {
            self.play_selected()?;
            Ok(EventState::Consumed)
        } else {
            Ok(EventState::NotConsumed)
        }
    }
}
