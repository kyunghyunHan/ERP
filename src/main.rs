use iced::{
    border::color,
    mouse,
    widget::{
        button,
        canvas::{
            self,
            event::{self, Event},
            Canvas, Program,
        },
        checkbox, column, container, pick_list, text, text_input, Checkbox, Column, Container,
        PickList, Row, Space, Text,
    },
    Background, Border, Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Subscription,
    Theme,
};
#[derive(Debug, Clone)]

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
        let top = Column::new().push(
            Row::new()
                .push(button(text("aa")).width(Length::FillPortion(1)))
                .push(button(text("aa")).width(Length::FillPortion(1)))
                .push(button(text("aa")).width(Length::FillPortion(1)))
                .push(button(text("aa")).width(Length::FillPortion(1))),
        );

        let middle = Container::new(Space::new(10, 10)).style(|_state| container::Style {
            // text_color: Some(Color::new(200., 1., 1., 1.)),
            background: Some(Background::Color(Color::from_rgb(255., 0., 0.))),
            border: Border::default(), // 이 부분은 다른 방식으로 수정해야 할 수 있습니다.
            ..container::Style::default()
        });
        Column::new().push(top).push(middle).into()
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
