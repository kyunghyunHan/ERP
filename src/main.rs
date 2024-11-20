use iced::{
    mouse,
    widget::{
        button, canvas,
        canvas::{
            event::{self, Event},
            Canvas, Program,
        },
        checkbox, column, container, pick_list, text, text_input, Checkbox, Column, Container,
        PickList, Space, Text,
    },
    Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Subscription, Theme,
};
#[derive(Debug)]

pub enum Message {
    ERROR,
}
#[derive(Debug)]
struct ERP {}
impl Default for ERP {
    fn default() -> Self {
        ERP {}
    }
}
impl ERP {
    pub fn view(&self) -> Element<Message> {
        Column::new().into()
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ERROR => {
                println!("{}", 1);
            }
        }
    }
}

fn main() -> iced::Result {
    iced::application("Candlestick Chart", ERP::update, ERP::view)
        .window_size(Size::new(1980., 1080.))
        .run()
}
