pub mod color {
    use iced::Color;

    pub const DARK_PURPLE: Color =
        Color::from_rgb(73.0 / 255.0, 65.0 / 255.0, 136.0 / 255.0);
    pub const LIGHT_PINK: Color =
        Color::from_rgb(1.0, 128.0 / 255.0, 238.0 / 255.0);
    pub const BLUE: Color = Color::from_rgb(0.0, 143.0 / 255.0, 214.0 / 255.0);
    pub const LIGHT_BLUE: Color =
        Color::from_rgb(120.0 / 255.0, 196.0 / 255.0, 1.0);
}

pub mod text {
    use iced::{font, Font};

    pub const BOLD: Font = Font {
        weight: font::Weight::Bold,
        ..Font::DEFAULT
    };
}

pub mod text_editor {
    use iced::border;
    use iced::widget::text_editor::{default, Status, Style};
    use iced::{Color, Font, Theme};

    pub const FONT: Font = Font::MONOSPACE;

    pub fn borderless(theme: &Theme, status: Status) -> Style {
        Style {
            border: border::color(Color::TRANSPARENT),
            ..default(theme, status)
        }
    }
}

pub mod icon {
    use iced::widget::svg::{Catalog, Handle, Status, Style, Svg};
    use iced::Theme;
    use std::sync::LazyLock;

    pub fn text(theme: &Theme, _status: Status) -> Style {
        Style {
            color: Some(theme.palette().text),
        }
    }

    pub fn info<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/info.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn warn<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/warning.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn danger<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/danger.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn trash<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/trash.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn filter<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/filter.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn close<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/close-x.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn bottom_layout<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/layout-bottom.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn left_layout<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/layout-left.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn right_layout<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/layout-right.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn hover_cursor<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/hover-cursor.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn inspector<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/inspector.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn terminal<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/terminal.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn new_window<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/new-window.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn chevron_down<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/chevron-down.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }

    pub fn chevron_right<'a, Theme: Catalog>() -> Svg<'a, Theme> {
        static HANDLE: LazyLock<Handle> = LazyLock::new(|| {
            Handle::from_memory(include_bytes!("../assets/chevron-right.svg"))
        });

        Svg::new(HANDLE.clone()).width(16).height(16)
    }
}
