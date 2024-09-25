use iced::advanced::widget;
use iced::advanced::widget::operation::inspectable;
use iced::widget::pane_grid;
use iced::widget::{canvas, container, stack, PaneGrid};
use iced::{
    window, Background, Color, Element, Fill, Size, Subscription, Task,
};

use tokio::sync::mpsc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod example;
use example::*;

mod tools;
use tools::*;

mod style;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

pub fn main() -> iced::Result {
    let (logger, receiver) = terminal::Logger::new();
    tracing_subscriber::registry()
        .with(
            logger.with_filter(
                EnvFilter::builder()
                    .with_default_directive(tracing::Level::TRACE.into())
                    .from_env()
                    .unwrap()
                    .add_directive("winit=debug".parse().unwrap())
                    .add_directive("iced_graphics=debug".parse().unwrap())
                    .add_directive("cosmic_text=info".parse().unwrap())
                    .add_directive("naga=info".parse().unwrap())
                    .add_directive("wgpu=warn".parse().unwrap()),
            ),
        )
        .init();

    iced::daemon("iced_devtools", Devtools::update, Devtools::view)
        .theme(Devtools::theme)
        .subscription(Devtools::subscription)
        .run_with(move || Devtools::new(receiver))
}

enum Pane<Content> {
    Content(Content),
    Devtools,
}

impl<Content> std::fmt::Debug for Pane<Content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pane::Content { .. } => f.write_str("Pane::Content"),
            Pane::Devtools => f.write_str("Pane::Devtools"),
        }
    }
}

enum Editor {
    Closed,
    Pane(pane_grid::Pane),
    Window(window::Id),
}

impl Editor {
    fn pane(&mut self) -> Option<&mut pane_grid::Pane> {
        if let Editor::Pane(pane) = self {
            return Some(pane);
        }

        None
    }
}

struct Devtools {
    content: pane_grid::Pane,
    window: window::Id,
    tools: Tools,
    editor: Editor,
    panes: pane_grid::State<Pane<Example>>,
    overlay: Option<inspector::Overlay>,
    element_selector: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Example(ExampleMessage),
    WindowResized,
    WindowClosed(window::Id),
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
            .split(pane_grid::Axis::Horizontal, content, Pane::Devtools)
            .map_or(Editor::Closed, |(pane, _)| Editor::Pane(pane));

        let (window, task) = window::open(window::Settings::default());

        (
            Self {
                window,
                content,
                editor,
                panes,
                tools: Tools::default(),
                overlay: None,
                element_selector: true,
            },
            Task::batch(vec![
                task.discard(),
                widget::operate(inspectable::map()).map(Message::Inspected),
                Task::run(terminal::run(receiver), |message| {
                    Message::Tools(tools::Message::Terminal(message))
                }),
            ]),
        )
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowClosed(id) => {
                if id == self.window {
                    return iced::exit();
                }
            }
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
                let (event, mut task) = self.tools.update(message);

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
                            .pane()
                            .map(|editor| self.panes.close(*editor));

                        self.overlay.take();
                    }
                    Some(Event::LayoutChanged(layout)) => {
                        match layout {
                            Layout::Windowed => {
                                match self.editor {
                                    Editor::Closed | Editor::Window(_) => {}
                                    Editor::Pane(editor) => {
                                        self.panes.close(editor);

                                        let (window, open) =
                                        window::open(window::Settings {
                                            size: Size::new(1024.0, 480.0),
                                            icon: window::icon::from_file_data(include_bytes!("../assets/iced.ico"), None).ok(),
                                            ..Default::default()
                                        });
                                        self.editor = Editor::Window(window);
                                        task = task.chain(open.discard());
                                    }
                                }
                            }
                            layout => match &mut self.editor {
                                Editor::Closed => {}
                                Editor::Window(id) => {
                                    task =
                                        task.chain(window::close(id.clone()));
                                    self.editor = self
                                        .panes
                                        .split(
                                            pane_grid::Axis::Horizontal,
                                            self.content,
                                            Pane::Devtools,
                                        )
                                        .and_then(|(pane, _)| {
                                            self.panes.move_to_edge(
                                                pane,
                                                layout.to_edge(),
                                            )
                                        })
                                        .map_or(Editor::Closed, |(pane, _)| {
                                            Editor::Pane(pane)
                                        });
                                }
                                Editor::Pane(editor) => {
                                    if let Some((pane, _)) = self
                                        .panes
                                        .move_to_edge(*editor, layout.to_edge())
                                    {
                                        *editor = pane;
                                    }
                                }
                            },
                        }

                        self.overlay.take();
                    }
                }

                return task.map(Message::Tools);
            }
        }

        iced::Task::none()
    }

    fn view(&self, id: window::Id) -> Element<Message> {
        match self.editor {
            Editor::Window(window) if id == window => {
                self.tools.view().map(Message::Tools)
            }
            _ => PaneGrid::new(&self.panes, |_, pane, _| match pane {
                Pane::Content(content) => {
                    let overlay = self.overlay.as_ref().map(|p| {
                        Element::from(canvas(p).width(Fill).height(Fill)).map(
                            |message| {
                                Message::Tools(tools::Message::Inspector(
                                    message,
                                ))
                            },
                        )
                    });

                    pane_grid::Content::new(
                        stack![content.view().map(Message::Example)]
                            .push_maybe(overlay),
                    )
                }
                Pane::Devtools => pane_grid::Content::new(
                    self.tools.view().map(Message::Tools),
                )
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgb(
                        0.15, 0.15, 0.15,
                    ))),
                    ..Default::default()
                }),
            })
            .on_resize(10, Message::PaneResized)
            .style(|theme| pane_grid::Style {
                hovered_split: pane_grid::Line {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                },
                picked_split: pane_grid::Line {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                },
                ..pane_grid::default(theme)
            })
            .into(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::resize_events().map(|(_, _)| Message::WindowResized),
            window::close_events().map(Message::WindowClosed),
        ])
    }

    fn theme(&self, _id: window::Id) -> iced::Theme {
        iced::Theme::Dark
    }
}
