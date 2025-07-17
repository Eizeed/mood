use crate::widget;

#[derive(Debug)]
pub enum Screen {
    Player(widget::Player),
    Playlist(),
}
