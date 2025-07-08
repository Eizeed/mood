use std::{io::BufReader, path::PathBuf, time::Duration};

use crossbeam_channel::{Receiver, Sender};
use rodio::{OutputStream, Sink, Source};

pub enum Command {
    Play(PathBuf),
    Pause,
    Resume,
    VolumeUp(f32),
    VolumeDown(f32),
    SeekForward(Duration),
    SeekBackward(Duration),
}

impl Command {
    pub fn play<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Command::Play(path.into())
    }

    pub fn resume() -> Self {
        Command::Resume
    }

    pub fn pause() -> Self {
        Command::Pause
    }
}

pub enum Message {
    TrackEnded,
    CurrentVolume(f32),
}

struct CustomSource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    inner: T,
    main_handle: Sender<Message>,
}

impl<T> Iterator for CustomSource<T>
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

impl<T> Source for CustomSource<T>
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
            let Ok(command) = rx.recv() else { return };
            match command {
                Command::Play(path) => {
                    let file = std::fs::File::open(path).unwrap();
                    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                    let source = CustomSource {
                        inner: source,
                        main_handle: tx.clone(),
                    };

                    // let _dur = source.total_duration();

                    sink.stop();
                    sink.play();
                    sink.append(source);
                }
                Command::Pause => {
                    sink.pause();
                }
                Command::Resume => {
                    sink.play();
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
                    sink.try_seek(pos + duration).unwrap();
                }
                Command::SeekBackward(duration) => {
                    let pos = sink.get_pos();
                    let new_pos = pos.saturating_sub(duration);
                    sink.try_seek(new_pos).unwrap();
                }
            }
        }
    });
}
