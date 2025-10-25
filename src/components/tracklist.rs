use std::path::PathBuf;

use color_eyre::Result;
use crossbeam_channel::Sender;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Paragraph};

use super::ComponentCommand;
use super::{Component, Widget, WidgetRef};
use crate::components::utils::VerticalScroll;
use crate::config::KeyConfig;
use crate::event::EventState;
use crate::models::Track;

pub struct TracklistComponent {
    library: Vec<Track>,
    scroll: VerticalScroll,
    key_config: KeyConfig,
    app_cmd_tx: Sender<ComponentCommand>,
}

pub enum Command {
    SetCurrentTrack { path: PathBuf },
}

impl TracklistComponent {
    pub fn new(
        lib: Vec<Track>,
        key_config: KeyConfig,
        app_cmd_tx: Sender<ComponentCommand>,
    ) -> Self {
        Self {
            library: lib,
            scroll: VerticalScroll::new(),
            key_config,
            app_cmd_tx,
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
        self.send_command(Command::SetCurrentTrack {
            path: path.to_path_buf(),
        })?;

        Ok(())
    }

    fn send_command(&self, cmd: Command) -> Result<()> {
        self.app_cmd_tx
            .send(ComponentCommand::TracklistComponent(cmd))?;
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
            let selection = self.scroll.pos() - self.scroll.y_offset.get();
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
