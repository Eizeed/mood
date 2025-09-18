mod player_controls;
mod playlist;
mod tracklist;

pub use player_controls::PlayerControlsComponent;
pub use playlist::PlaylistComponent;
pub use tracklist::TracklistComponent;

pub use ratatui::widgets::WidgetRef;
pub trait Component {
    fn event(&mut self, key: crate::event::Key);
}
