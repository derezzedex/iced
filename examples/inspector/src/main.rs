use iced::advanced::widget;
use iced::inspector;
use iced::inspector::widget::button;
use iced::widget::{canvas, column, stack, text, text_editor};
use iced::{highlighter, mouse, Center, Element, Fill, Task};

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application("A cool inspector", Inspector::update, Inspector::view)
        .theme(Inspector::theme)
        .subscription(Inspector::subscription)
        .run_with(Inspector::new)
}

#[derive(Default)]
struct Inspector {
    editor: Editor,
    overlay: Option<Overlay>,
    value: i64,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    WindowResized,
    Inspected(inspector::Map),
    Editor(EditorMessage),
}

impl Inspector {
    fn new() -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            widget::operate(inspector::map()).map(Message::Inspected),
        )
    }
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::WindowResized => {
                self.overlay.take();
                return widget::operate(inspector::map())
                    .map(Message::Inspected);
            }
            Message::Inspected(widgets) => {
                self.overlay = Some(Overlay { map: widgets });
            }
            Message::Editor(message) => {
                return self.editor.update(message).map(Message::Editor);
            }
        }

        iced::Task::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .width(Fill)
        .align_x(Center);

        let overlay = self
            .overlay
            .as_ref()
            .map(|p| canvas(p).width(Fill).height(Fill));

        column![
            stack![content].push_maybe(overlay),
            self.editor.view().map(Message::Editor),
        ]
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::window::resize_events().map(|(_, _)| Message::WindowResized)
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

struct Overlay {
    map: inspector::Map,
}

#[derive(Default)]
struct State {
    hovered: Option<inspector::Element>,
    cache: canvas::Cache,
}

const HIGHLIGHT: iced::Color = iced::Color::from_rgb(1.0, 0.0, 0.0);

impl canvas::Program<Message> for Overlay {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: canvas::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.hovered = self
                    .map
                    .widgets()
                    .filter(|el| el.bounds.contains(position))
                    .min_by(|a, b| a.size().partial_cmp(&b.size()).unwrap())
                    .cloned();
                state.cache.clear();

                return (
                    canvas::event::Status::Captured,
                    Some(Message::Editor(EditorMessage::Hovered(
                        state.hovered.clone(),
                    ))),
                );
            }
            _ => {}
        }

        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let frame = state.cache.draw(renderer, bounds.size(), |frame| {
            if let Some(hovered) = &state.hovered {
                highlight(hovered, frame);
            }
        });

        vec![frame]
    }
}

fn highlight(widget: &inspector::Element, frame: &mut canvas::Frame) {
    let path =
        canvas::Path::rectangle(widget.bounds.position(), widget.bounds.size());

    frame.stroke(
        &path,
        canvas::Stroke::default()
            .with_color(HIGHLIGHT)
            .with_width(2.0),
    );

    let padding = iced::Vector::new(1.0, 1.0);

    let content = widget.properties.name.clone();
    let content_width = content.len() as f32 * 7.5;

    let name = canvas::Text {
        content,
        position: widget.bounds.position() + padding,
        size: iced::Pixels(12.0),
        color: iced::Color::WHITE,
        font: iced::Font::MONOSPACE,
        ..Default::default()
    };

    frame.fill_text(name);
    frame.fill_rectangle(
        widget.bounds.position() + padding,
        iced::Size::new(content_width, 16.0),
        HIGHLIGHT,
    );
}

struct Editor {
    file: Option<String>,
    highlighted: Option<inspector::Element>,
    content: text_editor::Content,
    theme: highlighter::Theme,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            file: None,
            highlighted: None,
            content: text_editor::Content::new(),
            theme: highlighter::Theme::Base16Eighties,
        }
    }
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Hovered(Option<inspector::Element>),
    FileOpened(Arc<String>),
    EditorAction(text_editor::Action),
}

impl Editor {
    fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        match message {
            EditorMessage::Hovered(Some(element)) => {
                let file = element.properties.location.file().to_string();
                if self.highlighted.as_ref().is_some_and(|hl| {
                    hl.properties.location == element.properties.location
                }) {
                    return Task::none();
                }

                if self.file.as_ref().is_some_and(|current| current == &file) {
                    self.content.perform(text_editor::Action::Move(
                        text_editor::Motion::DocumentStart,
                    ));
                    let location = element.properties.location;

                    for _ in 1..location.line() {
                        self.content.perform(text_editor::Action::Move(
                            text_editor::Motion::Down,
                        ));
                    }

                    for _ in 0..location.column() {
                        self.content.perform(text_editor::Action::Move(
                            text_editor::Motion::Right,
                        ));
                    }

                    self.content.perform(text_editor::Action::SelectWord);

                    return Task::none();
                }

                self.file = Some(file.clone());

                return Task::perform(
                    open_file(file),
                    EditorMessage::FileOpened,
                );
            }
            EditorMessage::Hovered(None) => {
                self.highlighted = None;

                Task::none()
            }
            EditorMessage::EditorAction(action) => {
                if matches!(action, text_editor::Action::Scroll { .. }) {
                    self.content.perform(action);
                }

                Task::none()
            }
            EditorMessage::FileOpened(content) => {
                self.content = text_editor::Content::with_text(&content);

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<EditorMessage> {
        text_editor(&self.content)
            .font(iced::Font::MONOSPACE)
            .size(12)
            .on_action(EditorMessage::EditorAction)
            .highlight("rs", self.theme)
            .into()
    }
}

async fn open_file(path: impl Into<PathBuf>) -> Arc<String> {
    let path = Path::new(env!("CARGO_RUSTC_CURRENT_DIR")).join(path.into());
    tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .unwrap()
}
