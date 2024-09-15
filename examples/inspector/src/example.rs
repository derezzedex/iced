use iced::widget::{button, column, text};
use iced::{Center, Element, Fill};

#[derive(Default)]
pub struct Example {
    value: i64,
}

#[derive(Debug, Clone)]
pub enum ExampleMessage {
    Increment,
    Decrement,
}

impl Example {
    pub fn update(&mut self, message: ExampleMessage) {
        match message {
            ExampleMessage::Increment => {
                self.value += 1;
            }
            ExampleMessage::Decrement => {
                self.value -= 1;
            }
        }
    }

    pub fn view(&self) -> Element<ExampleMessage> {
        column![
            button("Increment")
                .padding(0)
                .on_press(ExampleMessage::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(ExampleMessage::Decrement)
        ]
        .padding(20)
        .width(Fill)
        .align_x(Center)
        .into()
    }
}
