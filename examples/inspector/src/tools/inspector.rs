use iced::advanced::widget;
use iced::advanced::widget::operation::inspectable;
use iced::widget::{
    button, canvas, column, container, horizontal_space, hover, row, rule,
    scrollable, text, text_editor, Column, Rule,
};
use iced::Alignment::Center;
use iced::{
    highlighter, mouse, Background, Element, Fill, Length, Padding, Task,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::style;
use crate::style::{color, icon};

#[derive(Debug, Clone)]
pub enum Section {
    Properties,
    Messages,
    Style,
}

#[derive(Clone, Copy)]
struct ExpandedSections {
    properties: bool,
    messages: bool,
    style: bool,
}

impl Default for ExpandedSections {
    fn default() -> Self {
        Self {
            properties: true,
            messages: false,
            style: false,
        }
    }
}

impl ExpandedSections {
    fn toggle(&mut self, section: Section) {
        match section {
            Section::Properties => self.properties = !self.properties,
            Section::Messages => self.messages = !self.messages,
            Section::Style => self.style = !self.style,
        }
    }
}

pub struct Inspector {
    file: Option<String>,
    locked: bool,
    properties: Option<text_editor::Content>,
    highlighted: Option<inspectable::Element>,
    content: text_editor::Content,
    theme: highlighter::Theme,
    expanded: ExpandedSections,
}

impl Default for Inspector {
    fn default() -> Self {
        Self {
            file: None,
            highlighted: None,
            properties: None,
            locked: false,
            content: text_editor::Content::new(),
            theme: highlighter::Theme::Base16Eighties,
            expanded: ExpandedSections::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Locked,
    Hovered(Option<inspectable::Element>),
    FileOpened(Arc<String>),
    EditorAction(text_editor::Action),
    TogglePropertiesEdit,
    PropertiesAction(text_editor::Action),
    SectionExpanded(Section),
}

impl Inspector {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SectionExpanded(section) => {
                self.expanded.toggle(section);

                Task::none()
            }
            Message::Locked => {
                self.locked = !self.locked;

                Task::none()
            }
            Message::TogglePropertiesEdit => {
                match self.properties.as_mut() {
                    None => {
                        if let Some(specific) = self
                            .highlighted
                            .as_ref()
                            .map(|el| &el.properties.specific)
                        {
                            self.properties =
                                specific.to_string_pretty().map(|s| {
                                    text_editor::Content::with_text(s.as_str())
                                });
                        }
                    }
                    Some(properties) => {
                        if let Some(element) = &self.highlighted {
                            let target = element.properties.id.clone();
                            let specific = properties.text();

                            return widget::operate(inspectable::edit(
                                target, specific,
                            ))
                            .discard();
                        }
                    }
                }

                Task::none()
            }
            Message::PropertiesAction(action) => {
                if let Some(properties) = &mut self.properties {
                    properties.perform(action);
                }

                Task::none()
            }
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
                    self.highlight_content_at(
                        location.line(),
                        location.column(),
                    );

                    return Task::none();
                }

                self.file = Some(file.clone());

                return Task::perform(open_file(file), Message::FileOpened);
            }
            Message::Hovered(None) => {
                if self.locked {
                    return Task::none();
                }

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

                if let Some(element) = &self.highlighted {
                    let location = element.properties.location;
                    self.highlight_content_at(
                        location.line(),
                        location.column(),
                    );
                }

                Task::none()
            }
        }
    }

    fn highlight_content_at(&mut self, line: u32, column: u32) {
        self.content.perform(text_editor::Action::Move(
            text_editor::Motion::DocumentStart,
        ));

        for _ in 1..line {
            self.content
                .perform(text_editor::Action::Move(text_editor::Motion::Down));
        }

        for _ in 0..column {
            self.content
                .perform(text_editor::Action::Move(text_editor::Motion::Right));
        }

        self.content.perform(text_editor::Action::SelectWord);
    }

    pub fn view(&self) -> Element<Message> {
        let text_editor = text_editor(&self.content)
            .height(Fill)
            .font(style::text_editor::FONT)
            .size(14)
            .on_action(Message::EditorAction)
            .wrapping(text::Wrapping::None)
            .style(style::text_editor::borderless)
            .highlight("rs", self.theme);

        let properties: iced::Element<Message> = if let Some(element) =
            &self.highlighted
        {
            scrollable(properties(
                self.properties.as_ref(),
                element,
                self.expanded,
            ))
            .spacing(2)
            .into()
        } else {
            container(
            column![
                        text!("Highlight a widget to see it's properties")
                        .size(12)
                            .font(style::text::BOLD),
                        row![
                            icon::hover_cursor().width(14).height(14).style(icon::text),
                            text!("You can toggle the widget hovering on the top left")
                                .size(12),
                        ].spacing(2).align_y(Center),
                    ]
                    .spacing(8)
                    .align_x(Center),
                )
                .center(Fill)
                .into()
        };

        row![
            container(text_editor).width(Length::FillPortion(2)),
            properties
        ]
        .spacing(4)
        .into()
    }
}

fn properties<'a>(
    properties: Option<&'a text_editor::Content>,
    element: &'a inspectable::Element,
    expanded: ExpandedSections,
) -> Element<'a, Message> {
    let content =
        properties.map_or(specific(&element.properties.specific), |editable| {
            text_editor(editable)
                .font(style::text_editor::FONT)
                .size(12)
                .wrapping(text::Wrapping::None)
                .on_action(Message::PropertiesAction)
                .highlight("json", highlighter::Theme::Base16Eighties)
                .style(style::text_editor::borderless)
                .into()
        });

    let properties = hover(
        container(content).width(Fill),
        container(
            button(if properties.is_some() { "Save" } else { "Edit" })
                .padding([2.0, 4.0])
                .on_press(Message::TogglePropertiesEdit)
                .style(button::primary),
        )
        .padding(5)
        .align_top(Fill)
        .align_right(Fill),
    )
    .into();

    column![
        row![
            text(element.properties.name.to_owned()).font(style::text::BOLD),
            horizontal_space(),
            text!("{:?}", element.properties.id).size(12),
        ]
        .padding(8),
        expandable(
            "Properties",
            expanded.properties.then_some(properties),
            Message::SectionExpanded(Section::Properties)
        ),
        expandable(
            "Messages",
            expanded
                .messages
                .then(|| specific(&element.properties.messages)),
            Message::SectionExpanded(Section::Messages)
        ),
        expandable(
            "Style",
            expanded.style.then(|| specific(&element.properties.style)),
            Message::SectionExpanded(Section::Style)
        ),
    ]
    .into()
}

fn expandable<'a>(
    title: impl Into<Element<'a, Message>>,
    content: Option<Element<'a, Message>>,
    toggle: Message,
) -> Element<'a, Message> {
    let rule = || {
        Rule::horizontal(1).style(|theme: &iced::Theme| rule::Style {
            color: theme.extended_palette().background.strong.color,
            width: 1,
            ..rule::default(theme)
        })
    };

    let chevron = if content.is_none() {
        icon::chevron_right().style(icon::text)
    } else {
        icon::chevron_down().style(icon::text)
    };

    let title = container(
        column![
            rule(),
            row![chevron, title.into()]
                .padding(4)
                .align_y(Center)
                .spacing(4),
        ]
        .push_maybe(content.is_some().then_some(rule())),
    )
    .width(Fill);

    column![button(title).padding(0).on_press(toggle).style(
        |theme: &iced::Theme, status| button::Style {
            background: if matches!(status, button::Status::Hovered) {
                Some(Background::Color(theme.palette().text.scale_alpha(0.05)))
            } else {
                None
            },
            ..button::text(theme, status)
        }
    )]
    .push_maybe(content)
    .into()
}

fn specific<'a, Message: 'a>(
    specific: &'a inspectable::Specific,
) -> Element<'a, Message> {
    Column::from_iter(specific.fields().iter().map(|(name, value)| {
        row![
            text(name).center().color(color::LIGHT_BLUE),
            text(
                inspectable::to_string_pretty(value)
                    .unwrap_or(String::from("None"))
            )
            .color(color::LIGHT_PINK)
        ]
        .spacing(8)
        .into()
    }))
    .padding([4.0, 8.0])
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
    locked: bool,
}

impl canvas::Program<Message> for Overlay {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if !cursor.is_over(bounds) {
            return (canvas::event::Status::Ignored, None);
        }

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )) => {
                state.locked = !state.locked;

                if !state.locked {
                    state.cache.clear();
                }

                return (
                    canvas::event::Status::Captured,
                    Some(Message::Locked),
                );
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if !self.hover_allowed || state.locked {
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
                        frame.fill(&quad, color::DARK_PURPLE.scale_alpha(0.8));
                    }
                }

                let mut blue_dashed = canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(color::BLUE);

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
