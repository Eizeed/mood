use std::{borrow::Cow, rc::Rc};

use ratatui::{
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::{component::Component, task::Task};

#[derive(Debug)]
pub struct Search<T>
where
    T: Searchable,
{
    list: Rc<[T]>,
    pattern: String,
    results: Vec<usize>,
    area: Rect,
}

pub enum Message {}

impl<T> Search<T>
where
    T: Searchable,
{
    pub fn new(list: Rc<[T]>, area: Rect) -> Self {
        Search {
            list,
            pattern: "".to_string(),
            results: vec![],
            area,
        }
    }
}

impl<T> Component for Search<T>
where
    T: Searchable,
{
    type Output = Task<Message>;
    type Message = Message;

    fn area(&self) -> Rect {
        self.area
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
    }

    fn update(&mut self, _message: Self::Message) -> Self::Output {
        Task::none()
    }

    fn view(&self, buffer: &mut ratatui::prelude::Buffer) {
        let outer = Block::bordered()
            .title("Search")
            .title_alignment(HorizontalAlignment::Center);

        let inner_area = outer.inner(self.area);

        let [search_area, content_area] = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .areas(inner_area);

        let search = Paragraph::new(&*self.pattern).block(Block::bordered());
        let content = Paragraph::new(if self.pattern == "" {
            Text::from_iter(self.list.iter().map(|item| Line::raw(item.name())))
        } else {
            Text::from_iter(
                self.list
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| self.results.contains(index))
                    .map(|(_, item)| Line::raw(item.name())),
            )
        })
        .block(Block::bordered());

        outer.render(self.area, buffer);
        search.render(search_area, buffer);
        content.render(content_area, buffer);
    }
}

pub trait Searchable {
    fn name(&self) -> Cow<'_, str>;
}
