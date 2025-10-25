use std::path::PathBuf;

use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::audio_thread::SinkState;

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

#[derive(Clone)]
pub enum Event {
    Tick,
    Input(Key),
    Audio(AudioMessage),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    Enter,
    Tab,
    Backspace,
    Esc,

    Left,
    Right,
    Up,
    Down,

    Insert,
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
        let mods = value.modifiers;
        let code = value.code;
        match code {
            KeyCode::Enter => Self::Enter,
            KeyCode::Tab => Self::Tab,
            KeyCode::Backspace => Self::Backspace,
            KeyCode::Esc => Self::Esc,

            KeyCode::Left => Self::Left,
            KeyCode::Right => Self::Right,
            KeyCode::Up => Self::Up,
            KeyCode::Down => Self::Down,

            KeyCode::Insert => Self::Insert,
            KeyCode::Delete => Self::Delete,
            KeyCode::Home => Self::Home,
            KeyCode::End => Self::End,
            KeyCode::PageUp => Self::PageUp,
            KeyCode::PageDown => Self::PageDown,

            KeyCode::F(0) => Self::F0,
            KeyCode::F(1) => Self::F1,
            KeyCode::F(2) => Self::F2,
            KeyCode::F(3) => Self::F3,
            KeyCode::F(4) => Self::F4,
            KeyCode::F(5) => Self::F5,
            KeyCode::F(6) => Self::F6,
            KeyCode::F(7) => Self::F7,
            KeyCode::F(8) => Self::F8,
            KeyCode::F(9) => Self::F9,
            KeyCode::F(10) => Self::F10,
            KeyCode::F(11) => Self::F11,
            KeyCode::F(12) => Self::F12,

            KeyCode::Char(c) if mods == KeyModifiers::CONTROL => Self::Ctrl(c),
            KeyCode::Char(c) if mods == KeyModifiers::ALT => Self::Alt(c),
            KeyCode::Char(c) => Self::Char(c),
            _ => Self::Unknown,
        }
    }
}

// TODO: Create actual messages
#[derive(Clone)]
pub enum AudioMessage {
    EndOfTrack,
    State(SinkState),
    Noop,
}

pub enum Command {
    Play(PathBuf),
    SendState,
    Noop,
}
