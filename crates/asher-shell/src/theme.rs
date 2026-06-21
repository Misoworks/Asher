use crate::color::Color;
use asher_config::AsherConfig;
use asher_material::{MaterialColor, shell_material_palette};

#[derive(Debug, Clone, Copy)]
pub struct ShellPalette {
    pub panel: Color,
    pub panel_control: Color,
    pub panel_text: Color,
    pub dock: Color,
    pub accent: Color,
    pub text_soft: Color,
    pub text_muted: Color,
}

impl Default for ShellPalette {
    fn default() -> Self {
        Self {
            panel: Color::rgba(22, 22, 20, 158),
            panel_control: Color::rgba(255, 255, 255, 20),
            panel_text: Color::rgba(248, 248, 246, 245),
            dock: Color::rgba(24, 23, 20, 86),
            accent: Color::rgba(210, 192, 130, 255),
            text_soft: Color::rgba(218, 216, 205, 232),
            text_muted: Color::rgba(164, 162, 154, 222),
        }
    }
}

pub fn shell_palette(config: &AsherConfig) -> ShellPalette {
    let palette = shell_material_palette(config);
    ShellPalette {
        panel: palette.panel.into(),
        panel_control: palette.panel_control.into(),
        panel_text: palette.panel_text.into(),
        dock: palette.dock.into(),
        accent: palette.accent.into(),
        text_soft: palette.text_soft.into(),
        text_muted: palette.text_muted.into(),
    }
}

impl From<MaterialColor> for Color {
    fn from(value: MaterialColor) -> Self {
        Color::rgba(value.r, value.g, value.b, value.a)
    }
}
