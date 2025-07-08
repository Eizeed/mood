use std::{fs::File, io::BufReader, time::Duration};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use rodio::{Decoder, OutputStream, Sink, Source};

pub enum Command {
    Play(Decoder<BufReader<File>>),
    Pause,
    Resume,

    VolumeUp(f32),
    VolumeDown(f32),

    SeekForward(Duration),
    SeekBackward(Duration),
}

impl Command {
    pub fn play<T>(source: T) -> Self
    where
        T: Into<Decoder<BufReader<File>>>,
    {
        Command::Play(source.into())
    }

    pub fn resume() -> Self {
        Command::Resume
    }

    pub fn pause() -> Self {
        Command::Pause
    }
}

#[derive(Debug)]
pub enum Message {
    TrackEnded,
    CurrentVolume(f32),
    CurrentPos(Duration),
}

struct NotifySource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    inner: T,
    main_handle: Sender<Message>,
}

impl<T> Iterator for NotifySource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();

        if let None = next {
            self.main_handle.send(Message::TrackEnded).unwrap();
        }

        next
    }
}

impl<T> Source for NotifySource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.inner.try_seek(pos)
    }
}

pub fn spawn_music(rx: Receiver<Command>, tx: Sender<Message>) {
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        tx.send(Message::CurrentVolume(sink.volume())).unwrap();

        loop {
            let command = match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(command) => command,
                Err(err) => match err {
                    RecvTimeoutError::Timeout => {
                        if !sink.empty() && !sink.is_paused() {
                            tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                        }
                        continue;
                    }
                    RecvTimeoutError::Disconnected => return,
                },
            };

            match command {
                Command::Play(source) => {
                    let source = NotifySource {
                        inner: source,
                        main_handle: tx.clone(),
                    };

                    sink.stop();
                    sink.play();
                    sink.append(source);
                }
                Command::Pause => {
                    sink.pause();
                }
                Command::Resume => {
                    sink.play();
                    tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                }
                Command::VolumeUp(vol) => {
                    if sink.volume() + vol >= 1.0 {
                        sink.set_volume(1.0);
                    } else {
                        sink.set_volume(sink.volume() + vol);
                    }

                    tx.send(Message::CurrentVolume(sink.volume())).unwrap();
                }
                Command::VolumeDown(vol) => {
                    if sink.volume() - vol <= 0.0 {
                        sink.set_volume(0.0);
                    } else {
                        sink.set_volume(sink.volume() - vol);
                    }

                    tx.send(Message::CurrentVolume(sink.volume())).unwrap();
                }
                Command::SeekForward(duration) => {
                    let pos = sink.get_pos();
                    sink.pause();
                    sink.try_seek(pos + duration).unwrap();
                    sink.play();

                    tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                }
                Command::SeekBackward(duration) => {
                    let pos = sink.get_pos();
                    let new_pos = pos.saturating_sub(duration);

                    sink.pause();
                    sink.try_seek(new_pos).unwrap();
                    sink.play();

                    tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                }
            }
        }
    });
}
