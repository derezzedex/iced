use crate::clipboard;
use crate::window;

pub enum Action<T> {
    Future(iced_futures::BoxFuture<T>),
    Clipboard(clipboard::Action<T>),
    Window(window::Action),
    // TODO: Possibly improve this, create Compositor(compositor::Action)?
    ReadFramebuffer {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        // TODO: Remove Vec<u8>
        message: Box<dyn Fn(Option<Vec<u8>>) -> T>,
    },
}

impl<T> Action<T> {
    /// Applies a transformation to the result of a [`Command`].
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + Send + Sync) -> Action<A>
    where
        T: 'static,
    {
        use iced_futures::futures::FutureExt;

        match self {
            Self::Future(future) => Action::Future(Box::pin(future.map(f))),
            Self::Clipboard(action) => Action::Clipboard(action.map(f)),
            Self::Window(window) => Action::Window(window),
            Self::ReadFramebuffer {
                x,
                y,
                width,
                height,
                message,
            } => Action::ReadFramebuffer {
                x,
                y,
                width,
                height,
                message: Box::new(move |s| f(message(s))),
            },
        }
    }
}
