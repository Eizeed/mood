use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    text::Text,
    widgets::{Block, Paragraph, Widget},
};

pub struct Popup {
    pub buffer: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    Push(char),
    Pop,
}

impl Popup {
    pub fn new() -> Self {
        Popup {
            buffer: String::new(),
        }
    }

    pub fn handle_input(&self, code: KeyCode, mods: KeyModifiers) -> Option<Message> {
        match code {
            KeyCode::Char(ch) => Some(Message::Push(ch)),
            KeyCode::Backspace => Some(Message::Pop),
            _ => None,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Message> {
        match message {
            Message::Push(ch) => self.buffer.push(ch),
            Message::Pop => {
                self.buffer.pop();
            }
        }

        None
    }
}

impl Widget for &Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(Text::raw(&self.buffer))
            .block(Block::bordered())
            .render(area, buf);
    }
}
