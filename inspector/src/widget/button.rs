use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::tree::{self, Tree};
use iced_core::widget::Operation;
use iced_core::{
    Clipboard, Element, Layout, Length, Padding, Rectangle, Shell, Size,
    Vector, Widget,
};
use iced_widget::button;

use core::panic::Location;

#[allow(missing_debug_implementations)]
pub struct Button<
    'a,
    Message,
    Theme = iced_widget::Theme,
    Renderer = iced_widget::Renderer,
> where
    Renderer: iced_core::Renderer,
    Theme: button::Catalog,
{
    inner: iced_widget::Button<'a, Message, Theme, Renderer>,
    properties: crate::Properties,
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Theme: button::Catalog,
    Message: std::fmt::Debug + Clone + 'a,
{
    /// Creates a new [`Button`] with the given content.
    #[track_caller]
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let inner = iced_widget::Button::new(content);
        let size = (&inner as &dyn Widget<Message, Theme, Renderer>).size();

        let properties = crate::Properties {
            name: String::from("Button"),
            location: Location::caller().clone(),
            padding: Some(Padding {
                // TODO: button::DEFAULT_PADDING is not pub
                top: 5.0,
                bottom: 5.0,
                right: 10.0,
                left: 10.0,
            }),
            size,
            handlers: Default::default(),
            clip: Some(false),
        };

        Self { inner, properties }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.properties.size.width = width;
        self.inner = self.inner.width(width);
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        let height = height.into();
        self.properties.size.height = height;
        self.inner = self.inner.height(height);
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        let padding = padding.into();
        self.properties.padding = Some(padding.clone());
        self.inner = self.inner.padding(padding);
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        let _ = self
            .properties
            .handlers
            .insert(String::from("on_press"), format!("{on_press:?}"));
        self.inner = self.inner.on_press(on_press);
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// This is analogous to [`Button::on_press`], but using a closure to produce
    /// the message.
    ///
    /// This closure will only be called when the [`Button`] is actually pressed and,
    /// therefore, this method is useful to reduce overhead if creating the resulting
    /// message is slow.
    pub fn on_press_with(
        mut self,
        on_press: impl Fn() -> Message + 'a,
    ) -> Self {
        let _ = self
            .properties
            .handlers
            .insert(String::from("on_press_with"), format!("{:?}", on_press()));
        self.inner = self.inner.on_press_with(on_press);
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        let _ = self
            .properties
            .handlers
            .insert(String::from("on_press_maybe"), format!("{on_press:?}"));
        self.inner = self.inner.on_press_maybe(on_press);
        self
    }

    /// Sets whether the contents of the [`Button`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.properties.clip = Some(clip);
        self.inner = self.inner.clip(clip);
        self
    }

    /// Sets the style of the [`Button`].
    #[must_use]
    pub fn style(
        mut self,
        style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
    ) -> Self
    where
        Theme::Class<'a>: From<button::StyleFn<'a, Theme>>,
    {
        self.inner = self.inner.style(style);
        self
    }

    /// Sets the style class of the [`Button`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.inner = self.inner.class(class);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer,
    Theme: button::Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<crate::State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(crate::State {
            properties: self.properties.clone(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            &self.inner as &dyn Widget<Message, Theme, Renderer>,
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        let tree = &mut tree.children[0];
        self.inner.diff(tree)
    }

    fn size(&self) -> Size<Length> {
        self.inner.size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let tree = &mut tree.children[0];
        self.inner.layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.custom(
            tree.state.downcast_mut::<crate::State>(),
            None,
            layout.bounds(),
        );

        operation.container(None, layout.bounds(), &mut |operation| {
            self.inner.operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let tree = &mut tree.children[0];
        self.inner.on_event(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let tree = &tree.children[0];
        self.inner
            .draw(&tree, renderer, theme, style, layout, cursor, viewport)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tree = &tree.children[0];
        self.inner
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let tree = &mut tree.children[0];
        self.inner.overlay(tree, layout, renderer, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for iced_core::Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: button::Catalog + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}
