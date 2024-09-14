use iced::advanced::widget;
use iced::advanced::widget::operation::inspectable;
use iced::widget::{button, horizontal_space, pane_grid, svg, Button};
use iced::widget::{
    canvas, column, container, row, scrollable, stack, text, text_editor,
    Column, PaneGrid, Svg,
};
use iced::{
    highlighter, mouse, Background, Center, Color, Element, Fill, Length,
    Padding, Task,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application("A cool inspector", Inspector::update, Inspector::view)
        .theme(Inspector::theme)
        .subscription(Inspector::subscription)
        .run_with(Inspector::new)
}

enum Pane<Content> {
    Content {
        content: Content,
        overlay: Option<Overlay>,
    },
    Editor(Editor),
}

impl<Content> std::fmt::Debug for Pane<Content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pane::Content { .. } => f.write_str("Pane::Content"),
            Pane::Editor(_) => f.write_str("Pane::Editor"),
        }
    }
}

#[derive(Default)]
struct Example {
    value: i64,
}

#[derive(Debug, Clone)]
enum ExampleMessage {
    Increment,
    Decrement,
}

impl Example {
    fn update(&mut self, message: ExampleMessage) {
        match message {
            ExampleMessage::Increment => {
                self.value += 1;
            }
            ExampleMessage::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<ExampleMessage> {
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

struct Inspector {
    content: pane_grid::Pane,
    editor: Option<pane_grid::Pane>,
    panes: pane_grid::State<Pane<Example>>,
}

#[derive(Debug, Clone)]
enum Message {
    Example(ExampleMessage),
    WindowResized,
    PaneResized(pane_grid::ResizeEvent),
    Inspected(inspectable::Map),
    Editor(EditorMessage),
}

impl Inspector {
    fn new() -> (Self, iced::Task<Message>) {
        let content = Pane::Content {
            content: Example::default(),
            overlay: None,
        };
        let (mut panes, content) = pane_grid::State::new(content);
        let editor = panes
            .split(
                pane_grid::Axis::Horizontal,
                content,
                Pane::Editor(Editor::default()),
            )
            .map(|(editor, _)| editor);

        (
            Self {
                content,
                editor,
                panes,
            },
            widget::operate(inspectable::map()).map(Message::Inspected),
        )
    }
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Example(message) => {
                if let Some(Pane::Content { content, .. }) =
                    self.panes.get_mut(self.content)
                {
                    content.update(message);
                }
            }
            Message::WindowResized => {
                return widget::operate(inspectable::map())
                    .map(Message::Inspected);
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
                return widget::operate(inspectable::map())
                    .map(Message::Inspected);
            }
            Message::Inspected(widgets) => {
                if let Pane::Content { overlay, .. } =
                    self.panes.get_mut(self.content).unwrap()
                {
                    *overlay = Some(Overlay { map: widgets });
                }
            }
            Message::Editor(message) => {
                if let Some(Pane::Editor(editor)) = self
                    .editor
                    .map(|editor| self.panes.get_mut(editor))
                    .flatten()
                {
                    let (event, task) = editor.update(message);
                    match event {
                        None => {}
                        Some(Event::Close) => {
                            self.editor
                                .take()
                                .map(|editor| self.panes.close(editor));

                            if let Pane::Content { overlay, .. } =
                                self.panes.get_mut(self.content).unwrap()
                            {
                                overlay.take();
                            }
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

                            if let Pane::Content { overlay, .. } =
                                self.panes.get_mut(self.content).unwrap()
                            {
                                overlay.take();
                            }
                        }
                    }

                    return task.map(Message::Editor);
                }
            }
        }

        iced::Task::none()
    }

    fn view(&self) -> Element<Message> {
        PaneGrid::new(&self.panes, |_, pane, _| match pane {
            Pane::Content { content, overlay } => {
                let overlay = overlay
                    .as_ref()
                    .map(|p| canvas(p).width(Fill).height(Fill));

                pane_grid::Content::new(
                    stack![content.view().map(Message::Example)]
                        .push_maybe(overlay),
                )
            }
            Pane::Editor(editor) => {
                pane_grid::Content::new(editor.view().map(Message::Editor))
                    .style(|_| container::Style {
                        background: Some(Background::Color(Color::from_rgb(
                            0.15, 0.15, 0.15,
                        ))),
                        ..Default::default()
                    })
            }
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

struct Overlay {
    map: inspectable::Map,
}

#[derive(Default)]
struct State {
    hovered: Option<inspectable::Element>,
    cache: canvas::Cache,
}

const DARK_PURPLE: Color =
    Color::from_rgb(73.0 / 255.0, 65.0 / 255.0, 136.0 / 255.0);
const LIGHT_PINK: Color = Color::from_rgb(1.0, 128.0 / 255.0, 238.0 / 255.0);
const BLUE: Color = Color::from_rgb(0.0, 143.0 / 255.0, 214.0 / 255.0);
const LIGHT_BLUE: Color = Color::from_rgb(120.0 / 255.0, 196.0 / 255.0, 1.0);

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

struct Editor {
    file: Option<String>,
    highlighted: Option<inspectable::Element>,
    content: text_editor::Content,
    theme: highlighter::Theme,
    layout: Layout,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            file: None,
            highlighted: None,
            content: text_editor::Content::new(),
            theme: highlighter::Theme::Base16Eighties,
            layout: Layout::Bottom,
        }
    }
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Hovered(Option<inspectable::Element>),
    FileOpened(Arc<String>),
    EditorAction(text_editor::Action),
    Close,
    ChangeLayout(Layout),
}

#[derive(Debug, Clone)]
enum Event {
    Close,
    LayoutChanged(Layout),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Layout {
    Bottom,
    Left,
    Right,
}

impl Layout {
    fn to_edge(&self) -> pane_grid::Edge {
        match self {
            Layout::Bottom => pane_grid::Edge::Bottom,
            Layout::Left => pane_grid::Edge::Left,
            Layout::Right => pane_grid::Edge::Right,
        }
    }
}

impl Editor {
    fn none() -> (Option<Event>, Task<EditorMessage>) {
        (None, Task::none())
    }

    fn task(task: Task<EditorMessage>) -> (Option<Event>, Task<EditorMessage>) {
        (None, task)
    }

    fn is_current_layout(&self, layout: Layout) -> bool {
        self.layout == layout
    }

    fn update(
        &mut self,
        message: EditorMessage,
    ) -> (Option<Event>, Task<EditorMessage>) {
        match message {
            EditorMessage::ChangeLayout(layout) => {
                self.layout = layout;
                (Some(Event::LayoutChanged(layout)), Task::none())
            }
            EditorMessage::Close => (Some(Event::Close), Task::none()),
            EditorMessage::Hovered(Some(element)) => {
                let file = element.properties.location.file().to_string();
                let location = element.properties.location;

                if self.highlighted.as_ref().is_some_and(|hl| {
                    hl.properties.location == element.properties.location
                }) {
                    return Self::none();
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

                    return Self::none();
                }

                self.file = Some(file.clone());

                return Self::task(Task::perform(
                    open_file(file),
                    EditorMessage::FileOpened,
                ));
            }
            EditorMessage::Hovered(None) => {
                self.highlighted = None;

                Self::none()
            }
            EditorMessage::EditorAction(action) => {
                if matches!(action, text_editor::Action::Scroll { .. }) {
                    self.content.perform(action);
                }

                Self::none()
            }
            EditorMessage::FileOpened(content) => {
                self.content = text_editor::Content::with_text(&content);

                Self::none()
            }
        }
    }

    fn view(&self) -> Element<EditorMessage> {
        let text_editor = text_editor(&self.content)
            .font(iced::Font::MONOSPACE)
            .size(12)
            .on_action(EditorMessage::EditorAction)
            .highlight("rs", self.theme);

        let properties: iced::Element<EditorMessage> = if let Some(element) =
            &self.highlighted
        {
            properties(element)
        } else {
            container(text!("Highlight some widget to see it's properties."))
                .padding(16)
                .into()
        };

        let title = container(text("Inspector").center()).padding(4);

        let controls = row![
            icon_button(
                bottom_layout().style(icon),
                self.is_current_layout(Layout::Bottom),
                EditorMessage::ChangeLayout(Layout::Bottom)
            ),
            icon_button(
                left_layout().style(icon),
                self.is_current_layout(Layout::Left),
                EditorMessage::ChangeLayout(Layout::Left)
            ),
            icon_button(
                right_layout().style(icon),
                self.is_current_layout(Layout::Right),
                EditorMessage::ChangeLayout(Layout::Right)
            ),
            horizontal_space().width(4),
            icon_button(close().style(icon), false, EditorMessage::Close),
        ];

        container(column![
            row![title, horizontal_space().width(Fill), controls],
            row![
                container(text_editor).width(Length::FillPortion(2)),
                scrollable(properties).width(Fill)
            ]
        ])
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
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

fn close<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/close-x.svg"
    )))
    .width(Length::Shrink)
    .height(Length::Shrink)
}

fn bottom_layout<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/layout-bottom.svg"
    )))
    .width(Length::Shrink)
    .height(Length::Shrink)
}

fn left_layout<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/layout-left.svg"
    )))
    .width(Length::Shrink)
    .height(Length::Shrink)
}

fn right_layout<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/layout-right.svg"
    )))
    .width(Length::Shrink)
    .height(Length::Shrink)
}

fn icon_button<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    active: bool,
    message: Message,
) -> Button<'a, Message> {
    button(content)
        .padding(2)
        .on_press(message)
        .style(move |theme, status| {
            let palette = theme.palette();

            let background = match status {
                button::Status::Active | button::Status::Hovered if active => {
                    Some(Background::Color(palette.text.scale_alpha(0.1)))
                }
                button::Status::Pressed => {
                    Some(Background::Color(palette.text.scale_alpha(0.1)))
                }
                _ => None,
            };

            button::Style {
                background,
                ..Default::default()
            }
        })
}

fn icon(theme: &iced::Theme, status: svg::Status) -> svg::Style {
    let palette = theme.palette();

    let color = match status {
        svg::Status::Idle => Some(palette.text),
        svg::Status::Hovered => Some(palette.primary),
    };

    svg::Style { color }
}

async fn open_file(path: impl Into<PathBuf>) -> Arc<String> {
    let path = Path::new(env!("CARGO_RUSTC_CURRENT_DIR")).join(path.into());
    tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .unwrap()
}
