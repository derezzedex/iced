use iced::futures::{self, SinkExt};
use iced::widget::{
    button, column, container, horizontal_space, row, rule, scrollable, svg,
    text, text_input, Column, Rule, Svg,
};
use iced::Alignment::Center;
use iced::Length::FillPortion;
use iced::{color, padding, stream, Background, Color};
use iced::{Element, Fill, Shrink, Task};

use std::collections::BTreeMap;

use chrono::{DateTime, Local};
use tokio::sync::mpsc;
use tracing::field::{Field, Visit};
use tracing_subscriber::registry::LookupSpan;

use std::sync::LazyLock;

use crate::style::{self, icon};

static SCROLLABLE_ID: LazyLock<scrollable::Id> =
    LazyLock::new(scrollable::Id::unique);

#[derive(Debug, Clone)]
struct LevelList {
    error: bool,
    warn: bool,
    info: bool,
    debug: bool,
    trace: bool,
}

impl Default for LevelList {
    fn default() -> Self {
        Self {
            error: true,
            warn: true,
            info: true,
            debug: false,
            trace: false,
        }
    }
}

impl LevelList {
    fn is_active(&self, level: tracing::Level) -> bool {
        match level {
            tracing::Level::ERROR => self.error,
            tracing::Level::WARN => self.warn,
            tracing::Level::INFO => self.info,
            tracing::Level::DEBUG => self.debug,
            tracing::Level::TRACE => self.trace,
        }
    }

    fn toggle(&mut self, level: tracing::Level) {
        let level = match level {
            tracing::Level::ERROR => &mut self.error,
            tracing::Level::WARN => &mut self.warn,
            tracing::Level::INFO => &mut self.info,
            tracing::Level::DEBUG => &mut self.debug,
            tracing::Level::TRACE => &mut self.trace,
        };

        *level = !*level;
    }
}

#[derive(Default)]
pub struct Terminal {
    filter: String,
    level: LevelList,
    logs: Vec<Log>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ReceivedLog(Log),
    Clear,
    FilterChanged(String),
    ToggleLevel(tracing::Level),
}

impl Terminal {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleLevel(level) => {
                self.level.toggle(level);

                Task::none()
            }
            Message::FilterChanged(filter) => {
                self.filter = filter;

                Task::none()
            }
            Message::Clear => {
                self.logs.clear();

                Task::none()
            }
            Message::ReceivedLog(log) => {
                let is_active = self.level.is_active(log.level);

                self.logs.push(log);

                if is_active {
                    return scrollable::snap_to(
                        SCROLLABLE_ID.clone(),
                        scrollable::RelativeOffset::END,
                    );
                }

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let logs: Vec<Element<Message>> = self
            .logs
            .iter()
            .filter(|log| {
                self.level.is_active(log.level)
                    && log
                        .fields
                        .values()
                        .any(|field| field.contains(&self.filter))
            })
            .map(|log| {
                container(column![
                    row![column![
                        row![
                            level_icon(log.level),
                            text(log.level.to_string())
                                .font(style::text::BOLD)
                                .size(14)
                        ]
                        .align_y(Center)
                        .spacing(4),
                        text(log.time.format("%H:%M:%S%.3f").to_string())
                            .size(12),
                    ]
                    .spacing(2),]
                    .height(Shrink)
                    .push_maybe(log.fields.get("message").map(|message| {
                        container(text(message).size(14).center())
                            .padding([0.0, 10.0])
                            .width(FillPortion(2))
                    }))
                    .push(horizontal_space())
                    .push(
                        column![
                            text(log.module.to_owned())
                                .size(12)
                                .font(style::text::BOLD),
                            text(log.location.to_owned()).size(12)
                        ]
                        .width(FillPortion(2))
                        .padding([0.0, 10.0]),
                    )
                    .padding(8)
                    .spacing(8),
                    Rule::horizontal(2).style(|theme| rule::Style {
                        color: level_color(log.level).scale_alpha(0.5),
                        ..rule::default(theme)
                    }),
                ])
                .width(Fill)
                .style(|_theme: &iced::Theme| container::Style {
                    text_color: Some(level_color(log.level)),
                    background: Some(iced::Background::Color(
                        level_color(log.level).scale_alpha(0.05),
                    )),
                    ..Default::default()
                })
                .into()
            })
            .collect();

        let controls = row![
            row![
                container(icon::filter().style(icon::text))
                    .align_y(Center)
                    .padding([4.0, 2.0]),
                text_input("Filter", &self.filter)
                    .size(12)
                    .on_input(Message::FilterChanged)
            ]
            .width(FillPortion(3)),
            row![
                level_button(tracing::Level::ERROR, self.level.error),
                level_button(tracing::Level::WARN, self.level.warn),
                level_button(tracing::Level::INFO, self.level.info),
                level_button(tracing::Level::DEBUG, self.level.debug),
                level_button(tracing::Level::TRACE, self.level.trace),
            ]
            .width(FillPortion(2)),
            button(icon::trash().style(icon::text))
                .padding(4)
                .style(button::text)
                .on_press(Message::Clear),
        ];

        let content: Element<Message> = if logs.len() == 0 {
            container(
                column![
                    text("No logs found").font(style::text::BOLD),
                    row![
                        icon::filter().width(14).height(14).style(icon::text),
                        text("You can tweak the filter options in the top bar")
                            .size(14)
                    ]
                    .spacing(4)
                    .align_y(Center)
                ]
                .align_x(Center)
                .spacing(8),
            )
            .center(Fill)
            .into()
        } else {
            scrollable(Column::from_vec(logs).width(Fill))
                .id(SCROLLABLE_ID.clone())
                .into()
        };

        column![controls, content,].into()
    }
}

fn level_button<'a>(
    level: tracing::Level,
    is_selected: bool,
) -> Element<'a, Message> {
    let icon =
        level_icon(level).style(move |theme: &iced::Theme, _| svg::Style {
            color: Some(active(theme, level, is_selected)),
        });

    let content = column![
        Rule::horizontal(2).style(move |theme: &iced::Theme| rule::Style {
            color: active(theme, level, is_selected),
            width: 2,
            ..rule::default(theme)
        },),
        row![
            icon.width(12).height(12),
            text(level.to_string()).size(12).center()
        ]
        .spacing(4)
        .padding(2)
        .align_y(Center),
    ];

    button(content)
        .padding(padding::all(2).top(0))
        .on_press(Message::ToggleLevel(level))
        .style(move |theme, status| {
            let color = active(theme, level, is_selected);

            let background = match status {
                button::Status::Active => {
                    if is_selected {
                        Some(Background::Color(color.scale_alpha(0.05)))
                    } else {
                        None
                    }
                }
                button::Status::Hovered => {
                    Some(Background::Color(color.scale_alpha(0.1)))
                }
                button::Status::Pressed => {
                    Some(Background::Color(color.scale_alpha(0.2)))
                }
                _ => None,
            };

            button::Style {
                background,
                text_color: color,
                ..Default::default()
            }
        })
        .into()
}

fn level_icon<'a>(level: tracing::Level) -> Svg<'a, iced::Theme> {
    level_svg(level)
        .width(14)
        .height(14)
        .style(move |_theme, _status| svg::Style {
            color: Some(level_color(level)),
        })
}

fn level_color(level: tracing::Level) -> iced::Color {
    match level {
        tracing::Level::TRACE => {
            color!(0x8ABEB7)
        }
        tracing::Level::DEBUG => {
            color!(0xB294BB)
        }
        tracing::Level::INFO => {
            color!(0x7E9EB9)
        }
        tracing::Level::WARN => {
            color!(0xF0C674)
        }
        tracing::Level::ERROR => {
            color!(0xCC6666)
        }
    }
}

fn active(
    theme: &iced::Theme,
    level: tracing::Level,
    is_active: bool,
) -> Color {
    let palette = theme.palette();

    if is_active {
        level_color(level)
    } else {
        palette.text
    }
}

fn level_svg<'a, Theme: svg::Catalog>(level: tracing::Level) -> Svg<'a, Theme> {
    match level {
        tracing::Level::TRACE
        | tracing::Level::DEBUG
        | tracing::Level::INFO => icon::info(),
        tracing::Level::WARN => icon::warn(),
        tracing::Level::ERROR => icon::danger(),
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub module: String,
    pub location: String,
    pub fields: BTreeMap<String, String>,
    pub level: tracing::Level,
    pub time: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct Logger {
    sender: mpsc::Sender<Log>,
}

impl Logger {
    pub fn new() -> (Self, mpsc::Receiver<Log>) {
        let (sender, receiver) = mpsc::channel(100);

        (Self { sender }, receiver)
    }
}

impl<S> tracing_subscriber::Layer<S> for Logger
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use tracing_log::NormalizeEvent;

        let normalized = event.normalized_metadata();
        let metadata = normalized.as_ref().unwrap_or_else(|| event.metadata());

        let mut fields = BTreeMap::new();
        event.record(&mut FieldVisitor(&mut fields));

        let module = metadata.module_path().unwrap_or("unknown").to_owned();
        let file = metadata.file().unwrap_or("unknown").to_owned();
        let line_number = metadata.line().unwrap_or(0).to_owned();

        let _ = self.sender.try_send(Log {
            // callsite: metadata.file().map_or(String::from("unknown"), |file| String::from(file)),
            module,
            location: format!("{file}:{line_number}"),
            time: Local::now(),
            level: metadata.level().to_owned(),
            fields,
        });
    }
}

pub fn run(
    mut receiver: mpsc::Receiver<Log>,
) -> impl futures::Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        while let Some(log) = receiver.recv().await {
            let _ = output.send(Message::ReceivedLog(log)).await;
        }
    })
}

struct FieldVisitor<'a>(&'a mut BTreeMap<String, String>);

impl<'a> Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}
