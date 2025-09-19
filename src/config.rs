use std::path::PathBuf;

use crate::event::Key;

#[derive(Default)]
pub struct Config {
    pub audio_dir: PathBuf,
    pub key_config: KeyConfig,
}

impl Config {
    pub fn new(audio_dir: PathBuf) -> Self {
        Config {
            audio_dir,
            key_config: KeyConfig::default(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct KeyConfig {
    pub quit: Key,

    pub scroll_up: Key,
    pub scroll_down: Key,

    pub play_audio: Key,
    pub add_to_manual_queue: Key,

    pub skip_to_next_audio: Key,
    pub skip_to_prev_audio: Key,
    pub seek_forward: Key,
    pub seek_backward: Key,

    pub pause: Key,
    pub shuffle: Key,
    pub repeat: Key,

    pub pick_playlist: Key,

    pub create_playlist: Key,
    pub delete_playlist: Key,

    pub focus_playlist_popup: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        KeyConfig {
            quit: Key::Esc,
            scroll_up: Key::Char('k'),
            scroll_down: Key::Char('j'),
            play_audio: Key::Enter,
            add_to_manual_queue: Key::Char('q'),
            skip_to_next_audio: Key::Char('l'),
            skip_to_prev_audio: Key::Char('h'),
            seek_forward: Key::Ctrl('l'),
            seek_backward: Key::Ctrl('h'),
            pause: Key::Char(' '),
            shuffle: Key::Char('s'),
            repeat: Key::Char('r'),
            pick_playlist: Key::Enter,
            create_playlist: Key::Enter,
            delete_playlist: Key::Char('D'),
            focus_playlist_popup: Key::Char('p'),
        }
    }
}
