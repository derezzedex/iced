use iced::widget::{
    button, column, container, horizontal_space, pane_grid, row, svg, text,
    Button, Svg,
};
use iced::{Background, Color, Element, Fill, Shrink, Task};

pub mod inspector;
pub use inspector::*;

#[derive(Debug, Clone)]
pub enum Kind {
    Inspector { hover_allowed: bool },
}

impl Kind {
    pub fn is_hover_allowed(&self) -> bool {
        match self {
            Kind::Inspector { hover_allowed } => *hover_allowed,
        }
    }

    fn toggle_hover(&mut self) {
        match self {
            Self::Inspector { hover_allowed } => {
                *hover_allowed = !*hover_allowed
            }
        }
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::Inspector {
            hover_allowed: true,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Layout {
    #[default]
    Bottom,
    Left,
    Right,
}

impl Layout {
    pub fn to_edge(&self) -> pane_grid::Edge {
        match self {
            Layout::Bottom => pane_grid::Edge::Bottom,
            Layout::Left => pane_grid::Edge::Left,
            Layout::Right => pane_grid::Edge::Right,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Close,
    LayoutChanged(Layout),
    HoverToggled,
}

#[derive(Default)]
pub struct Tools {
    selected: Kind,
    layout: Layout,
    inspector: Inspector,
}

#[derive(Debug, Clone)]
pub enum Message {
    Inspector(inspector::Message),
    Close,
    ChangeLayout(Layout),
    ToggleHover,
}

impl Tools {
    pub fn update(
        &mut self,
        message: Message,
    ) -> (Option<Event>, Task<Message>) {
        match message {
            Message::Inspector(message) => {
                (None, self.inspector.update(message).map(Message::Inspector))
            }
            Message::ToggleHover => {
                self.selected.toggle_hover();

                (Some(Event::HoverToggled), Task::none())
            }
            Message::ChangeLayout(layout) => {
                self.layout = layout;
                (Some(Event::LayoutChanged(layout)), Task::none())
            }
            Message::Close => (Some(Event::Close), Task::none()),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let element_selector = icon_button(
            element_selector().style(icon),
            self.selected.is_hover_allowed(),
            Message::ToggleHover,
        )
        .padding([2.0, 4.0]);

        let title = container(text("Inspector").center()).padding([0.0, 4.0]);

        let controls = row![
            icon_button(
                bottom_layout().style(icon),
                self.layout == Layout::Bottom,
                Message::ChangeLayout(Layout::Bottom)
            ),
            icon_button(
                left_layout().style(icon),
                self.layout == Layout::Left,
                Message::ChangeLayout(Layout::Left)
            ),
            icon_button(
                right_layout().style(icon),
                self.layout == Layout::Right,
                Message::ChangeLayout(Layout::Right)
            ),
            horizontal_space().width(4),
            icon_button(close().style(icon), false, Message::Close),
        ];

        let content = match self.selected {
            Kind::Inspector { .. } => {
                self.inspector.view().map(Message::Inspector)
            }
        };

        container(column![
            row![
                element_selector,
                title,
                horizontal_space().width(Fill),
                controls
            ],
            content,
        ])
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
    }
}

fn close<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/close-x.svg"
    )))
    .width(Shrink)
    .height(Shrink)
}

fn bottom_layout<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/layout-bottom.svg"
    )))
    .width(Shrink)
    .height(Shrink)
}

fn left_layout<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/layout-left.svg"
    )))
    .width(Shrink)
    .height(Shrink)
}

fn right_layout<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/layout-right.svg"
    )))
    .width(Shrink)
    .height(Shrink)
}

fn element_selector<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    Svg::new(svg::Handle::from_memory(include_bytes!(
        "../assets/element-selector.svg"
    )))
    .width(Shrink)
    .height(Shrink)
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
