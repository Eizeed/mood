use std::io::BufReader;
use std::time::Duration;

use color_eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use rodio::Source;

use crate::event::{AudioMessage, Command, Event};

pub struct AudioThread {
    command_rx: Receiver<Command>,
    event_tx: Sender<Event>,
    source_total_duraiton: Option<Duration>,
}

impl AudioThread {
    pub fn new(command_rx: Receiver<Command>, event_tx: Sender<Event>) -> AudioThread {
        AudioThread {
            command_rx,
            event_tx,
            source_total_duraiton: None,
        }
    }

    pub fn run(mut self) -> Result<()> {
        _ = std::thread::spawn(move || -> Result<()> {
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());
            sink.set_volume(0.05);

            loop {
                // Accept command
                let cmd = self.command_rx.recv()?;

                match cmd {
                    Command::Play(path) => {
                        let file = std::fs::File::open(&path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        let notify_source = NotifySource {
                            inner: source,
                            main_handle: self.event_tx.clone(),
                        };

                        self.source_total_duraiton = notify_source.total_duration();

                        sink.clear();
                        sink.append(notify_source);
                        sink.play();
                    }
                    Command::SendState => {
                        let state = SinkState {
                            total_duraiton: self.source_total_duraiton.clone(),
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
    pub total_duraiton: Option<Duration>,
    pub pos: Duration,
    pub volume: f32,
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
