use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    text::Text,
    widgets::{Block, Paragraph, Widget},
};

use crate::{task::Task, widget::Component};

pub struct Popup {
    pub buffer: String,
    area: Rect,
}

#[derive(Clone, Debug)]
pub enum Message {
    Push(char),
    Pop,
}

impl Popup {
    pub fn new(area: Rect) -> Self {
        Popup {
            buffer: String::new(),
            area,
        }
    }

}

impl Component for Popup {
    type Message = Message;
    type Output = Task<Message>;
    fn area(&self) -> Rect {
        self.area
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
    }

    fn update(&mut self, message: Self::Message) -> Self::Output {
        match message {
            Message::Push(ch) => self.buffer.push(ch),
            Message::Pop => {
                self.buffer.pop();
            }
        }

        Task::none()
    }

    fn handle_input(&self, code: KeyCode, _mods: KeyModifiers) -> Option<Message> {
        match code {
            KeyCode::Char(ch) => Some(Message::Push(ch)),
            KeyCode::Backspace => Some(Message::Pop),
            _ => None,
        }
    }
    
    fn view(&self, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(Text::raw(&self.buffer))
            .block(Block::bordered())
            .render(self.area, buf);
    }
}
