use std::time::Duration;

use color_eyre::Result;
use crossbeam_channel::Sender;
use crossterm::event;

use crate::event::{Event, Key};

pub fn spawn_event_emmiter(
    event_tx: Sender<Event>,
    tickrate: Duration,
) -> Result<()> {
    _ = std::thread::spawn(move || -> Result<()> {
        loop {
            if event::poll(tickrate)? {
                if let event::Event::Key(key) = event::read()? {
                    let key = Key::from(key);
                    event_tx.send(Event::Input(key))?;
                }
            }

            event_tx.send(Event::Tick)?;
        }
    });

    Ok(())
}
