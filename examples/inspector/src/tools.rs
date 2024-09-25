use std::fmt;

use iced::alignment::Vertical;
use iced::widget::{
    button, column, container, horizontal_space, pane_grid, row, rule, svg,
    text, Button, Rule, Svg,
};
use iced::Alignment::Center;
use iced::{Background, Color, Element, Fill, Font, Shrink, Task};

pub mod inspector;
pub use inspector::*;

pub mod terminal;
pub use terminal::Terminal;

use crate::style::icon;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Kind {
    #[default]
    Inspector,
    Terminal,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Inspector => "Inspector",
            Kind::Terminal => "Terminal",
        })
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Layout {
    #[default]
    Bottom,
    Left,
    Right,
    Windowed,
}

impl Layout {
    pub fn to_edge(&self) -> pane_grid::Edge {
        match self {
            Layout::Bottom => pane_grid::Edge::Bottom,
            Layout::Left => pane_grid::Edge::Left,
            Layout::Right => pane_grid::Edge::Right,
            Layout::Windowed => pane_grid::Edge::Bottom,
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
    hover_cursor: bool,
    layout: Layout,
    inspector: Inspector,
    terminal: Terminal,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeTab(Kind),
    Inspector(inspector::Message),
    Terminal(terminal::Message),
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
            Message::Terminal(message) => {
                (None, self.terminal.update(message).map(Message::Terminal))
            }
            Message::Inspector(message) => {
                (None, self.inspector.update(message).map(Message::Inspector))
            }
            Message::ChangeTab(selected) => {
                self.selected = selected;
                (None, Task::none())
            }
            Message::ToggleHover => {
                self.hover_cursor = !self.hover_cursor;

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
        let hover_cursor = icon_button(
            icon::hover_cursor(),
            self.hover_cursor,
            Message::ToggleHover,
        );

        let tabs = row![
            tab_button(Kind::Inspector, &self.selected,),
            tab_button(Kind::Terminal, &self.selected,),
        ];

        let controls = row![
            icon_button(
                icon::bottom_layout(),
                self.layout == Layout::Bottom,
                Message::ChangeLayout(Layout::Bottom)
            ),
            icon_button(
                icon::left_layout(),
                self.layout == Layout::Left,
                Message::ChangeLayout(Layout::Left)
            ),
            icon_button(
                icon::right_layout(),
                self.layout == Layout::Right,
                Message::ChangeLayout(Layout::Right)
            ),
            icon_button(
                icon::new_window(),
                self.layout == Layout::Windowed,
                Message::ChangeLayout(Layout::Windowed)
            ),
            icon_button(icon::close(), false, Message::Close),
        ];

        let content = match self.selected {
            Kind::Inspector { .. } => {
                self.inspector.view().map(Message::Inspector)
            }
            Kind::Terminal => self.terminal.view().map(Message::Terminal),
        };

        container(column![
            Rule::horizontal(1).style(|theme: &iced::Theme| rule::Style {
                color: theme.extended_palette().background.base.color,
                width: 1,
                ..rule::default(theme)
            }),
            row![hover_cursor, tabs, horizontal_space(), controls],
            content,
            column![
                Rule::horizontal(1).style(|theme: &iced::Theme| rule::Style {
                    color: theme.extended_palette().background.base.color,
                    width: 1,
                    ..rule::default(theme)
                }),
                row![
                    horizontal_space(),
                    icon::iced().content_fit(iced::ContentFit::Cover),
                    text(iced::VERSION).font(Font::MONOSPACE).size(10),
                    horizontal_space().width(4)
                ]
                .align_y(Center)
                .spacing(4),
            ]
            .width(Fill)
        ])
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
    }
}

fn tab_button(kind: Kind, selected: &Kind) -> Element<Message> {
    let is_selected = kind == *selected;

    let icon = match kind {
        Kind::Terminal => icon::terminal(),
        Kind::Inspector { .. } => icon::inspector(),
    }
    .style(move |theme, _| icon(theme, is_selected));

    let content = column![
        Rule::horizontal(2).style(move |theme: &iced::Theme| rule::Style {
            color: active(theme, is_selected),
            width: 2,
            ..rule::default(theme)
        },),
        row![icon, text(kind.to_string()).size(14).center()]
            .spacing(4)
            .padding(4)
            .align_y(Vertical::Center),
    ]
    .width(Shrink);

    button(content)
        .padding(0)
        .on_press(Message::ChangeTab(kind))
        .style(move |theme, status| {
            let palette = theme.palette();

            let background = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(palette.text.scale_alpha(0.05)))
                }
                _ => None,
            };

            button::Style {
                background,
                text_color: active(theme, is_selected),
                ..Default::default()
            }
        })
        .into()
}

fn icon_button<'a, Message: 'a>(
    content: Svg<'a>,
    is_active: bool,
    message: Message,
) -> Button<'a, Message> {
    button(content.style(move |theme, _| icon(theme, is_active)))
        .padding(2)
        .on_press(message)
        .style(move |theme, status| {
            let palette = theme.palette();

            let background = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(palette.text.scale_alpha(0.05)))
                }
                _ => None,
            };

            button::Style {
                background,
                text_color: active(theme, is_active),
                ..Default::default()
            }
        })
}

fn active(theme: &iced::Theme, is_active: bool) -> Color {
    let palette = theme.palette();

    if is_active {
        palette.primary
    } else {
        palette.text
    }
}

fn icon(theme: &iced::Theme, is_active: bool) -> svg::Style {
    svg::Style {
        color: Some(active(theme, is_active)),
    }
}
