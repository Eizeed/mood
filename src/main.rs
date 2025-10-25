use std::{path::PathBuf, time::Duration};

use rusqlite::Connection;

use crate::{
    app::App, audio_thread::AudioThread, config::Config, event::Event, utils::spawn_event_emmiter,
};

mod app;
mod audio_thread;
mod components;
mod config;
mod event;
mod io;
mod models;
mod utils;

fn main() -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();

    let tickrate = Duration::from_millis(250);

    let (event_tx, event_rx) = crossbeam_channel::unbounded();
    let (command_tx, command_rx) = crossbeam_channel::unbounded();

    spawn_event_emmiter(event_tx.clone(), tickrate)?;
    AudioThread::new(command_rx, event_tx).run()?;

    let config = Config::new(PathBuf::from("/home/lf/music"));

    let sqlite = Connection::open("db.db3")?;

    let mut app = App::new(command_tx, config, sqlite)?;

    terminal.draw(|f| app.render(f.area(), f.buffer_mut()))?;
    loop {
        match event_rx.recv()? {
            Event::Input(key) => {
                if !app.event(key)?.is_consumed() {
                    if key == app.config.key_config.quit {
                        break;
                    }
                }
            }
            Event::Tick => {
                app.tick();
            }
            Event::Audio(audio) => {
                app.audio(audio);
            }
        }

        terminal.draw(|f| app.render(f.area(), f.buffer_mut()))?;
    }

    ratatui::restore();

    Ok(())
}
