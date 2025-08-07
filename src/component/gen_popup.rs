use std::ops::{Deref, DerefMut};

use ratatui::{
    layout::Rect,
    widgets::{Clear, Widget},
};

use crate::component::Component;

pub struct Popup<T>
where
    T: Component,
{
    inner: T,
    show: bool,
}

impl<T> Popup<T>
where
    T: Component,
{
    pub fn new(inner: T) -> Self {
        Popup { inner, show: false }
    }

    pub fn show(&mut self) {
        self.show = true;
    }

    pub fn hide(&mut self) {
        self.show = false;
    }
}

impl<T: Component> Deref for Popup<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Component> DerefMut for Popup<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Component> Component for Popup<T> {
    type Output = T::Output;
    type Message = T::Message;

    fn area(&self) -> Rect {
        self.inner.area()
    }

    fn resize(&mut self, area: Rect) {
        self.inner.resize(area);
    }

    fn update(&mut self, message: Self::Message) -> Self::Output {
        self.inner.update(message)
    }

    fn handle_input(
        &self,
        code: crossterm::event::KeyCode,
        mods: crossterm::event::KeyModifiers,
    ) -> Option<Self::Message> {
        self.inner.handle_input(code, mods)
    }

    fn handle_mouse(&self, ev: crossterm::event::MouseEvent) -> Option<Self::Message> {
        self.inner.handle_mouse(ev)
    }

    fn view(&self, buffer: &mut ratatui::prelude::Buffer) {
        Clear::render(Clear, self.area(), buffer);
        if self.show {
            self.inner.view(buffer);
        }
    }
}
