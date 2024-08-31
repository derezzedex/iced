pub mod button;
pub use button::Button;

/// Creates a new [`Button`] with the provided content.
///
/// [`Button`]: crate::Button
#[track_caller]
pub fn button<'a, Message, Theme, Renderer>(
    content: impl Into<iced_core::Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: iced_widget::button::Catalog + 'a,
    Renderer: iced_core::Renderer + 'a,
    Message: std::fmt::Debug + Clone + 'a,
{
    Button::new(content)
}

pub use iced_core::widget::*;
