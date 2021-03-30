use crate::{Settings, Viewport};
use glow::HasContext;
use iced_graphics::backend;
use iced_graphics::font;
use iced_graphics::Primitive;
use iced_native::mouse;
use iced_native::{Font, Size};

mod embedded;
mod compatibility;
mod modern;

/// A [`glow`] graphics backend for [`iced`].
///
/// [`glow`]: https://github.com/grovesNL/glow
/// [`iced`]: https://github.com/hecrj/iced
#[derive(Debug)]
pub struct Backend {
    mode: Mode,
    default_text_size: u16,
}

#[derive(Debug)]
enum Mode{
    Modern(modern::Backend),
    Compatibility(compatibility::Backend),
    Embedded(embedded::Backend),
}

impl Backend {
    /// Creates a new [`Backend`].
    pub fn new(gl: &glow::Context, settings: Settings) -> Self {
        unsafe {
            println!("Vendor: {}", gl.get_parameter_string(glow::VENDOR));
            println!("Renderer: {}", gl.get_parameter_string(glow::RENDERER));
            println!("Version: {}", gl.get_parameter_string(glow::VERSION));
            println!("GLSL Version: {}", gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION));
        }
        let mode = Mode::Embedded(embedded::Backend::new(gl, settings));

        Self{
            mode,
            default_text_size: settings.default_text_size,
        }
    }

    /// Draws the provided primitives in the default framebuffer.
    ///
    /// The text provided as overlay will be rendered on top of the primitives.
    /// This is useful for rendering debug information.
    pub fn draw<T: AsRef<str>>(
        &mut self,
        gl: &glow::Context,
        viewport: &Viewport,
        primitive_interaction: &(Primitive, mouse::Interaction),
        overlay_text: &[T],
    ) -> mouse::Interaction {
        match &mut self.mode{
            Mode::Modern(backend) => backend.draw(gl, viewport, primitive_interaction, overlay_text),
            Mode::Compatibility(backend) => backend.draw(gl, viewport, primitive_interaction, overlay_text),
            Mode::Embedded(backend) => backend.draw(gl, viewport, primitive_interaction, overlay_text),
        }
    }
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        match &mut self.mode{
            Mode::Modern(backend) => backend.trim_measurements(),
            Mode::Compatibility(backend) => backend.trim_measurements(),
            Mode::Embedded(backend) => backend.trim_measurements(),
        }
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = font::ICONS;
    const CHECKMARK_ICON: char = font::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = font::ARROW_DOWN_ICON;

    fn default_size(&self) -> u16 {
        self.default_text_size
    }

    fn measure(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        match &self.mode{
            Mode::Modern(backend) => backend.measure(contents, size, font, bounds),
            Mode::Compatibility(backend) => backend.measure(contents, size, font, bounds),
            Mode::Embedded(backend) => backend.measure(contents, size, font, bounds),
        }
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(&self, _handle: &iced_native::image::Handle) -> (u32, u32) {
        (50, 50)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        _handle: &iced_native::svg::Handle,
    ) -> (u32, u32) {
        (50, 50)
    }
}
