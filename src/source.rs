use std::time::Duration;

use crossbeam_channel::Sender;
use rodio::Source;

use crate::event::{AudioMessage, Event};

pub struct NotifySource<T>
where
    T: Source,
{
    pub inner: T,
    pub app_event_tx: Sender<Event>,
}

impl<T> NotifySource<T>
where
    T: Source,
{
    pub fn new(source: T, app_event_tx: Sender<Event>) -> Self {
        NotifySource {
            inner: source,
            app_event_tx,
        }
    }
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
                .app_event_tx
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
