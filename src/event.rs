use std::time::Duration;

use crossterm::event;

#[derive(PartialEq, Debug)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == EventState::Consumed
    }
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            EventState::Consumed
        } else {
            EventState::NotConsumed
        }
    }
}

pub struct EventHandler {
    rx: crossbeam_channel::Receiver<Event>,
}

impl EventHandler {
    pub fn new(tickrate: Duration) -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        _ = std::thread::spawn(move || {
            loop {
                if event::poll(tickrate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        let key = Key::from(key);
                        event_tx.send(Event::Input(key)).unwrap();
                    }
                }

                event_tx.send(Event::Tick).unwrap();
            }
        });

        EventHandler { rx: event_rx }
    }

    pub fn next(&self) -> Result<Event, crossbeam_channel::RecvError> {
        self.rx.recv()
    }
}

#[derive(Copy, Clone)]
pub enum Event {
    Tick,
    Input(Key),
    Audio(AudioMessage),
}

#[derive(Copy, Clone)]
pub enum Key {
    Enter,
    Tab,
    Backspace,
    Esc,

    Left,
    Right,
    Up,
    Down,

    Ins,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    F0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Char(char),
    Ctrl(char),
    Alt(char),
    Unknown,
}

impl From<event::KeyEvent> for Key {
    fn from(value: event::KeyEvent) -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy)]
pub enum AudioMessage {}

#[derive(Clone, Copy)]
pub enum Command {}
