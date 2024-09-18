use iced::futures::{self, SinkExt};
use iced::widget::{
    column, container, horizontal_space, row, rule, scrollable, svg, text,
    Column, Rule, Svg,
};
use iced::Alignment::Center;
use iced::Length::FillPortion;
use iced::{color, stream};
use iced::{Element, Fill, Shrink, Task};

use std::collections::BTreeMap;

use chrono::{DateTime, Local};
use tokio::sync::mpsc;
use tracing::field::{Field, Visit};
use tracing_subscriber::registry::LookupSpan;

use std::sync::LazyLock;

static SCROLLABLE_ID: LazyLock<scrollable::Id> =
    LazyLock::new(scrollable::Id::unique);

#[derive(Default)]
pub struct Terminal {
    logs: Vec<Log>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ReceivedLog(Log),
}

impl Terminal {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ReceivedLog(log) => {
                self.logs.push(log);

                return scrollable::snap_to(
                    SCROLLABLE_ID.clone(),
                    scrollable::RelativeOffset::END,
                );
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let logs = self.logs.iter().map(|log| {
            container(column![
                row![column![
                    row![
                        level_icon(log.level),
                        text(log.level.to_string().to_lowercase())
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(14)
                    ]
                    .align_y(Center)
                    .spacing(4),
                    text(log.time.format("%H:%M:%S%.3f").to_string()).size(12),
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
                        text(log.module.to_owned()).size(12).font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }),
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
        });

        scrollable(Column::from_iter(logs).width(Fill))
            .id(SCROLLABLE_ID.clone())
            .into()
    }
}

fn level_icon<'a>(level: tracing::Level) -> Element<'a, Message> {
    level_svg(level)
        .width(14)
        .height(14)
        .style(move |_theme, _status| svg::Style {
            color: Some(level_color(level)),
        })
        .into()
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

fn level_svg<'a, Theme: svg::Catalog>(level: tracing::Level) -> Svg<'a, Theme> {
    match level {
        tracing::Level::TRACE => info(),
        tracing::Level::DEBUG => info(),
        tracing::Level::INFO => info(),
        tracing::Level::WARN => warning(),
        tracing::Level::ERROR => danger(),
    }
}

fn info<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    static INFO_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
        svg::Handle::from_memory(include_bytes!("../../assets/info.svg"))
    });

    Svg::new(INFO_SVG.clone()).width(16).height(16)
}

fn warning<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    static WARNING_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
        svg::Handle::from_memory(include_bytes!("../../assets/warning.svg"))
    });

    Svg::new(WARNING_SVG.clone()).width(16).height(16)
}

fn danger<'a, Theme: svg::Catalog>() -> Svg<'a, Theme> {
    static DANGER_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
        svg::Handle::from_memory(include_bytes!("../../assets/danger.svg"))
    });

    Svg::new(DANGER_SVG.clone()).width(16).height(16)
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
