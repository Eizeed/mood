use color_eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use std::time::Duration;

use crate::{
    event::{AudioMessage, Command, Event},
    source::NotifySource,
};

pub struct AudioThread {
    command_rx: Receiver<Command>,
    event_tx: Sender<Event>,
}

impl AudioThread {
    pub fn new(command_rx: Receiver<Command>, event_tx: Sender<Event>) -> AudioThread {
        AudioThread {
            command_rx,
            event_tx,
        }
    }

    pub fn run(self) -> Result<()> {
        _ = std::thread::spawn(move || -> Result<()> {
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());
            sink.set_volume(0.05);

            loop {
                // Accept command
                let cmd = self.command_rx.recv()?;

                match cmd {
                    Command::Play(source) => {
                        let notify_source = NotifySource::new(*source, self.event_tx.clone());
                        sink.clear();
                        sink.append(notify_source);
                        sink.play();
                    }
                    Command::SendState => {
                        let state = SinkState {
                            pos: sink.get_pos(),
                            volume: sink.volume(),
                        };

                        _ = self.event_tx.send(Event::Audio(AudioMessage::State(state)));
                    }
                    Command::Noop => {
                        continue;
                    }
                }

                // Emmit event
                self.event_tx.send(Event::Audio(AudioMessage::Noop))?;
            }
        });

        Ok(())
    }
}

#[derive(Clone)]
pub struct SinkState {
    pub pos: Duration,
    pub volume: f32,
}
