use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    text::Text,
    widgets::{Block, Paragraph, Widget},
};

use crate::{action::Action, widget::Component};

pub struct Popup {
    pub buffer: String,
    area: Rect,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Submit(String),
    Cancel,
}

#[derive(Clone, Debug)]
pub enum Message {
    Push(char),
    Pop,
    Cancel,
    Submit,
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
    type Output = Action<Instruction, Message>;

    fn area(&self) -> Rect {
        self.area
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
    }

    fn update(&mut self, message: Self::Message) -> Self::Output {
        match message {
            Message::Push(ch) => {
                self.buffer.push(ch);
                Action::none()
            }
            Message::Pop => {
                self.buffer.pop();
                Action::none()
            }
            Message::Cancel => {
                self.buffer.clear();
                Action::instruction(Instruction::Cancel)
            }
            Message::Submit => {
                if self.buffer.is_empty() {
                    return Action::none();
                }

                Action::instruction(Instruction::Submit(std::mem::take(&mut self.buffer)))
            }
        }
    }

    fn handle_input(&self, code: KeyCode, mods: KeyModifiers) -> Option<Message> {
        match code {
            KeyCode::Char('c') if mods == KeyModifiers::CONTROL => Some(Message::Cancel),
            KeyCode::Enter => Some(Message::Submit),
            KeyCode::Backspace => Some(Message::Pop),
            KeyCode::Char(ch) => Some(Message::Push(ch)),
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
