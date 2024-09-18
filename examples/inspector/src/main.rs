use iced::advanced::widget;
use iced::advanced::widget::operation::inspectable;
use iced::widget::pane_grid;
use iced::widget::{canvas, container, stack, PaneGrid};
use iced::{Background, Color, Element, Fill, Task};

use tokio::sync::mpsc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod example;
use example::*;

mod tools;
use tools::*;
use tracing_subscriber::Layer;

pub fn main() -> iced::Result {
    let (logger, receiver) = terminal::Logger::new();
    tracing_subscriber::registry()
        .with(logger.with_filter(tracing::level_filters::LevelFilter::INFO))
        .init();

    iced::application("A cool inspector", Devtools::update, Devtools::view)
        .theme(Devtools::theme)
        .subscription(Devtools::subscription)
        .run_with(move || Devtools::new(receiver))
}

enum Pane<Content> {
    Content(Content),
    Tools(Tools),
}

impl<Content> std::fmt::Debug for Pane<Content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pane::Content { .. } => f.write_str("Pane::Content"),
            Pane::Tools(_) => f.write_str("Pane::Devtools"),
        }
    }
}

struct Devtools {
    content: pane_grid::Pane,
    editor: Option<pane_grid::Pane>,
    panes: pane_grid::State<Pane<Example>>,
    overlay: Option<inspector::Overlay>,
    element_selector: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Example(ExampleMessage),
    WindowResized,
    PaneResized(pane_grid::ResizeEvent),
    Inspected(inspectable::Map),
    Tools(tools::Message),
}

impl Devtools {
    fn new(
        receiver: mpsc::Receiver<terminal::Log>,
    ) -> (Self, iced::Task<Message>) {
        let content = Pane::Content(Example::default());
        let (mut panes, content) = pane_grid::State::new(content);
        let editor = panes
            .split(
                pane_grid::Axis::Horizontal,
                content,
                Pane::Tools(Tools::default()),
            )
            .map(|(editor, _)| editor);

        (
            Self {
                content,
                editor,
                panes,
                overlay: None,
                element_selector: true,
            },
            Task::batch(vec![
                widget::operate(inspectable::map()).map(Message::Inspected),
                Task::run(terminal::run(receiver), |message| {
                    Message::Tools(tools::Message::Terminal(message))
                }),
            ]),
        )
    }
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Example(message) => {
                if let Some(Pane::Content(content)) =
                    self.panes.get_mut(self.content)
                {
                    content.update(message);
                }
            }
            Message::WindowResized => {
                if !self.element_selector {
                    return Task::none();
                }

                return widget::operate(inspectable::map())
                    .map(Message::Inspected);
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);

                if !self.element_selector {
                    return Task::none();
                }

                return widget::operate(inspectable::map())
                    .map(Message::Inspected);
            }
            Message::Inspected(map) => {
                self.overlay = Some(Overlay::new(map, !self.element_selector));
            }
            Message::Tools(message) => {
                if let Some(Pane::Tools(editor)) = self
                    .editor
                    .map(|editor| self.panes.get_mut(editor))
                    .flatten()
                {
                    let (event, task) = editor.update(message);
                    match event {
                        None => {}
                        Some(Event::HoverToggled) => {
                            self.element_selector = !self.element_selector;
                            if let Some(overlay) = &mut self.overlay {
                                overlay.hover_allowed = !overlay.hover_allowed;
                            }
                        }
                        Some(Event::Close) => {
                            self.editor
                                .take()
                                .map(|editor| self.panes.close(editor));

                            self.overlay.take();
                        }
                        Some(Event::LayoutChanged(layout)) => {
                            if let Some(editor) = &mut self.editor {
                                if let Some((pane, _)) = self
                                    .panes
                                    .move_to_edge(*editor, layout.to_edge())
                                {
                                    *editor = pane;
                                }
                            }

                            self.overlay.take();
                        }
                    }

                    return task.map(Message::Tools);
                }
            }
        }

        iced::Task::none()
    }

    fn view(&self) -> Element<Message> {
        PaneGrid::new(&self.panes, |_, pane, _| match pane {
            Pane::Content(content) => {
                let overlay = self.overlay.as_ref().map(|p| {
                    Element::from(canvas(p).width(Fill).height(Fill)).map(
                        |message| {
                            Message::Tools(tools::Message::Inspector(message))
                        },
                    )
                });

                pane_grid::Content::new(
                    stack![content.view().map(Message::Example)]
                        .push_maybe(overlay),
                )
            }
            Pane::Tools(tools) => pane_grid::Content::new(
                tools.view().map(Message::Tools),
            )
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(
                    0.15, 0.15, 0.15,
                ))),
                ..Default::default()
            }),
        })
        .on_resize(10, Message::PaneResized)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::window::resize_events().map(|(_, _)| Message::WindowResized)
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
