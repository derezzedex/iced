use iced::advanced::widget::{self, operation};
use iced::widget::{button, column, text, Column};
use iced::Center;
use iced::Task;

pub fn main() -> iced::Result {
    iced::run("A cool counter", Counter::update, Counter::view)
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    Inspected(operation::inspectable::Map),
}

impl Counter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::Inspected(foo) => {
                println!("foo: {foo:#?}");
                
                return Task::none();
            }
        }

        widget::operate(operation::inspectable::map())
            .map(Message::Inspected)
    }

    fn view(&self) -> Column<Message> {
        column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .align_x(Center)
    }
}
