use iced::advanced::widget;
use iced::advanced::widget::operation::inspectable;
use iced::widget::{button, canvas, column, stack, text};
use iced::{Center, Element};

pub fn main() -> iced::Result {
    iced::application("A cool inspector", Inspector::update, Inspector::view)
        .subscription(Inspector::subscription)
        .run_with(Inspector::new)
}

#[derive(Default)]
struct Inspector {
    overlay: Option<Overlay>,
    value: i64,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    WindowResized,
    Inspected(inspectable::Map),
}

impl Inspector {
    fn new() -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            widget::operate(inspectable::map()).map(Message::Inspected),
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
                return widget::operate(inspectable::map())
                    .map(Message::Inspected);
            }
            Message::Inspected(widgets) => {
                self.overlay = Some(Overlay {
                    map: widgets,
                    cache: canvas::Cache::new(),
                });
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
        .align_x(Center);

        let overlay = self
            .overlay
            .as_ref()
            .map(|p| canvas(p).width(iced::Fill).height(iced::Fill));

        stack![content].push_maybe(overlay).into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::window::resize_events().map(|(_, _)| Message::WindowResized)
    }
}

struct Overlay {
    map: inspectable::Map,
    cache: canvas::Cache,
}

const HIGHLIGHT: iced::Color = iced::Color::from_rgb(1.0, 0.0, 0.0);

impl canvas::Program<Message> for Overlay {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        if let Some(mouse) = cursor.position() {
            let hovered = self
                .map
                .widgets()
                .filter(|el| el.bounds.contains(mouse))
                .min_by(|a, b| a.size().partial_cmp(&b.size()).unwrap());

            if let Some(hovered) = hovered {
                highlight(hovered, &mut frame);
            }
        }

        vec![frame.into_geometry()]
    }
}

fn highlight(widget: &inspectable::Element, frame: &mut canvas::Frame) {
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
