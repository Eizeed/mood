use std::time::Duration;

use color_eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use crossterm::event;

use crate::event::{AudioMessage, Command, Event, Key};

pub fn spawn_event_emmiter(event_tx: Sender<Event>, tickrate: Duration) -> Result<()> {
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

pub fn spawn_audio_thread(
    command_rx: Receiver<Command>,
    event_tx: Sender<Event>,
) -> color_eyre::Result<()> {
    _ = std::thread::spawn(move || -> Result<()> {
        loop {
            // Accept command
            command_rx.recv()?;

            // Emmit event
            event_tx.send(Event::Audio(AudioMessage::Noop))?;
        }
    });

    Ok(())
}
