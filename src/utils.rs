use std::{io::BufReader, time::Duration};

use color_eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use crossterm::event;
use rodio::Source;

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
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        sink.set_volume(0.05);

        loop {
            // Accept command
            let cmd = command_rx.recv()?;

            match cmd {
                Command::Play(path) => {
                    let file = std::fs::File::open(&path).unwrap();
                    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                    let notify_source = NotifySource {
                        inner: source,
                        main_handle: event_tx.clone(),
                    };

                    sink.clear();
                    sink.append(notify_source);
                    sink.play();
                }
                Command::Noop => {
                    continue;
                }
            }

            // Emmit event
            event_tx.send(Event::Audio(AudioMessage::Noop))?;
        }
    });

    Ok(())
}

struct NotifySource<T>
where
    T: Source,
{
    inner: T,
    main_handle: Sender<Event>,
}

impl<T> Iterator for NotifySource<T>
where
    T: Source,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.inner.next();
        if n.is_none() {
            _ = self
                .main_handle
                .send(Event::Audio(AudioMessage::EndOfTrack));
        }

        n
    }
}

impl<T> Source for NotifySource<T>
where
    T: Source,
{
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
