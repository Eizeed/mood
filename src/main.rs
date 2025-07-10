use crate::{app::App, io::get_config, widget::Player};

mod app;
mod config;
mod input;
mod io;
mod music;
mod widget;

fn main() {
    let terminal = ratatui::init();

    let config = get_config();

    let player = Player::new(config.audio_dir);

    let app = App::with_player(player);
    app.start(terminal);

    ratatui::restore();
}
