use iced::{Element, Task};

pub mod inspector;
pub use inspector::*;

#[derive(Default, Debug, Clone)]
pub enum Kind {
    #[default]
    Inspector,
}

#[derive(Default)]
pub struct Tools {
    selected: Kind,
    inspector: Inspector,
}

#[derive(Debug, Clone)]
pub enum Message {
    Inspector(inspector::Message),
}

impl Tools {
    pub fn update(
        &mut self,
        message: Message,
    ) -> (Option<inspector::Event>, Task<Message>) {
        match message {
            Message::Inspector(message) => {
                let (event, task) = self.inspector.update(message);
                (event, task.map(Message::Inspector))
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match self.selected {
            Kind::Inspector => self.inspector.view().map(Message::Inspector),
        }
    }
}
