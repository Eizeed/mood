pub mod player_controls;
pub mod playlist;
pub mod tracklist;
pub mod utils;

pub use player_controls::PlayerControlsComponent;
pub use playlist::PlaylistComponent;
pub use tracklist::TracklistComponent;

use color_eyre::Result;
pub use ratatui::widgets::Widget;
pub use ratatui::widgets::WidgetRef;

use crate::event::EventState;

pub trait Component {
    fn event(&mut self, key: crate::event::Key) -> Result<EventState>;
}

pub enum ComponentCommand {
    TracklistComponent(tracklist::Command)
}
