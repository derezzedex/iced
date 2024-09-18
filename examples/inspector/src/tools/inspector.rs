use iced::advanced::widget::operation::inspectable;
use iced::widget::{
    canvas, column, container, row, scrollable, text, text_editor, Column,
};
use iced::{highlighter, mouse, Color, Element, Fill, Length, Padding, Task};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Inspector {
    file: Option<String>,
    highlighted: Option<inspectable::Element>,
    content: text_editor::Content,
    theme: highlighter::Theme,
}

impl Default for Inspector {
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
pub enum Message {
    Hovered(Option<inspectable::Element>),
    FileOpened(Arc<String>),
    EditorAction(text_editor::Action),
}

impl Inspector {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Hovered(Some(element)) => {
                let file = element.properties.location.file().to_string();
                let location = element.properties.location;

                if self.highlighted.as_ref().is_some_and(|hl| {
                    hl.properties.location == element.properties.location
                }) {
                    return Task::none();
                }

                self.highlighted = Some(element);

                if self.file.as_ref().is_some_and(|current| current == &file) {
                    self.content.perform(text_editor::Action::Move(
                        text_editor::Motion::DocumentStart,
                    ));

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

                return Task::perform(open_file(file), Message::FileOpened);
            }
            Message::Hovered(None) => {
                self.highlighted = None;

                Task::none()
            }
            Message::EditorAction(action) => {
                if matches!(action, text_editor::Action::Scroll { .. }) {
                    self.content.perform(action);
                }

                Task::none()
            }
            Message::FileOpened(content) => {
                self.content = text_editor::Content::with_text(&content);

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let text_editor = text_editor(&self.content)
            .font(iced::Font::MONOSPACE)
            .size(12)
            .on_action(Message::EditorAction)
            .highlight("rs", self.theme);

        let properties: iced::Element<Message> = if let Some(element) =
            &self.highlighted
        {
            properties(element)
        } else {
            container(text!("Highlight some widget to see it's properties."))
                .padding(16)
                .into()
        };

        row![
            container(text_editor).width(Length::FillPortion(2)),
            scrollable(properties).width(Fill)
        ]
        .into()
    }
}

fn properties<'a, Message: 'a>(
    element: &'a inspectable::Element,
) -> Element<'a, Message> {
    let specific = element.properties.specific.fields();

    column![
        text("Properties").size(18),
        Column::from_iter(specific.iter().map(|(name, value)| {
            row![
                text(name).center().color(LIGHT_BLUE),
                text(
                    inspectable::to_string_pretty(value)
                        .unwrap_or(String::from("None"))
                )
                .color(LIGHT_PINK)
            ]
            .padding(4)
            .spacing(8)
            .into()
        })),
    ]
    .spacing(8)
    .padding(16)
    .into()
}

async fn open_file(path: impl Into<PathBuf>) -> Arc<String> {
    let path = Path::new(env!("CARGO_RUSTC_CURRENT_DIR")).join(path.into());
    tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .unwrap()
}

pub struct Overlay {
    map: inspectable::Map,
    pub hover_allowed: bool,
}

impl Overlay {
    pub fn new(map: inspectable::Map, locked: bool) -> Self {
        Self {
            map,
            hover_allowed: locked,
        }
    }
}

#[derive(Default)]
pub struct State {
    hovered: Option<inspectable::Element>,
    cache: canvas::Cache,
}

pub const DARK_PURPLE: Color =
    Color::from_rgb(73.0 / 255.0, 65.0 / 255.0, 136.0 / 255.0);
pub const LIGHT_PINK: Color =
    Color::from_rgb(1.0, 128.0 / 255.0, 238.0 / 255.0);
pub const BLUE: Color = Color::from_rgb(0.0, 143.0 / 255.0, 214.0 / 255.0);
pub const LIGHT_BLUE: Color =
    Color::from_rgb(120.0 / 255.0, 196.0 / 255.0, 1.0);

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
                if !self.hover_allowed {
                    return (canvas::event::Status::Ignored, None);
                }

                state.hovered = self
                    .map
                    .widgets()
                    .filter(|el| el.bounds.contains(position))
                    .min_by(|a, b| a.size().partial_cmp(&b.size()).unwrap())
                    .cloned();
                state.cache.clear();

                return (
                    canvas::event::Status::Captured,
                    Some(Message::Hovered(state.hovered.clone())),
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
                if let Some(padding) =
                    hovered.properties.specific.find_and_get::<Padding>()
                {
                    for quad in padding_quads(hovered.bounds, padding) {
                        frame.fill(&quad, DARK_PURPLE.scale_alpha(0.8));
                    }
                }

                let mut blue_dashed =
                    canvas::Stroke::default().with_width(1.0).with_color(BLUE);

                blue_dashed.line_dash = canvas::stroke::LineDash {
                    segments: &[5.0, 3.0],
                    offset: 0,
                };

                for line in bounding_lines(bounds, hovered.bounds) {
                    frame.stroke(&line, blue_dashed);
                }
            }
        });

        vec![frame]
    }
}

fn padding_quads(
    element: iced::Rectangle,
    padding: iced::Padding,
) -> [canvas::Path; 4] {
    let bottom = element.position()
        + iced::Vector::new(0.0, element.height - padding.bottom);
    let left = element.position() + iced::Vector::new(0.0, padding.top);
    let right = element.position()
        + iced::Vector::new(element.width - padding.right, padding.top);

    [
        canvas::Path::rectangle(
            element.position(),
            iced::Size::new(element.width, padding.top),
        ),
        canvas::Path::rectangle(
            bottom,
            iced::Size::new(element.width, padding.bottom),
        ),
        canvas::Path::rectangle(
            left,
            iced::Size::new(
                padding.left,
                element.height - padding.top - padding.bottom,
            ),
        ),
        canvas::Path::rectangle(
            right,
            iced::Size::new(
                padding.right,
                element.height - padding.top - padding.bottom,
            ),
        ),
    ]
}

fn bounding_lines(
    bounds: iced::Rectangle,
    element: iced::Rectangle,
) -> [canvas::Path; 4] {
    [
        canvas::Path::line(
            iced::Point::new(element.x, bounds.y),
            iced::Point::new(element.x, bounds.height),
        ),
        canvas::Path::line(
            iced::Point::new(element.x + element.width, bounds.y),
            iced::Point::new(element.x + element.width, bounds.height),
        ),
        canvas::Path::line(
            iced::Point::new(bounds.x, element.y),
            iced::Point::new(bounds.width, element.y),
        ),
        canvas::Path::line(
            iced::Point::new(bounds.x, element.y + element.height),
            iced::Point::new(bounds.width, element.y + element.height),
        ),
    ]
}
