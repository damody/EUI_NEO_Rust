use crate::color::Color;

/// Theme color set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub background: Color,
    pub primary: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub text: Color,
    pub border: Color,
}

pub fn light_theme() -> Theme {
    Theme {
        background: Color::new(0.95, 0.95, 0.97, 1.0),
        primary: Color::new(0.20, 0.50, 0.90, 1.0),
        surface: Color::new(1.00, 1.00, 1.00, 1.0),
        surface_hover: Color::new(0.90, 0.90, 0.90, 1.0),
        surface_active: Color::new(0.80, 0.80, 0.80, 1.0),
        text: Color::new(0.00, 0.00, 0.00, 1.0),
        border: Color::new(0.80, 0.80, 0.80, 1.0),
    }
}

pub fn dark_theme() -> Theme {
    Theme {
        background: Color::new(0.10, 0.10, 0.12, 1.0),
        primary: Color::new(0.30, 0.60, 1.00, 1.0),
        surface: Color::new(0.15, 0.15, 0.18, 1.0),
        surface_hover: Color::new(0.25, 0.25, 0.28, 1.0),
        surface_active: Color::new(0.35, 0.35, 0.38, 1.0),
        text: Color::new(1.00, 1.00, 1.00, 1.0),
        border: Color::new(0.30, 0.30, 0.30, 1.0),
    }
}

/// Extended theme color tokens (matches C++ ThemeColorTokens).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColorTokens {
    pub background: Color,
    pub primary: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub text: Color,
    pub border: Color,
    pub dark: bool,
}

/// Page visual tokens (matches C++ PageVisualTokens).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageVisualTokens {
    pub title_color: Color,
    pub subtitle_color: Color,
    pub body_color: Color,
    pub card_color: Color,
    pub muted_card_color: Color,
    pub soft_accent_color: Color,
    pub header_top_inset: f32,
    pub header_title_gap: f32,
    pub header_content_gap: f32,
    pub header_title_size: f32,
    pub header_subtitle_size: f32,
    pub section_gap: f32,
    pub section_inset: f32,
    pub section_rounding: f32,
    pub label_size: f32,
    pub field_height: f32,
}

/// Field visual tokens (matches C++ UIFieldVisualTokens).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIFieldVisualTokens {
    pub rounding: f32,
    pub horizontal_inset: f32,
    pub focus_line_height: f32,
    pub border_line_height: f32,
    pub popup_rounding: f32,
    pub popup_overlap: f32,
    pub popup_shadow_color: Color,
    pub popup_shadow_blur: f32,
    pub popup_shadow_offset_y: f32,
}

impl Default for UIFieldVisualTokens {
    fn default() -> Self {
        Self {
            rounding: 6.0,
            horizontal_inset: 10.0,
            focus_line_height: 2.0,
            border_line_height: 1.0,
            popup_rounding: 10.0,
            popup_overlap: 1.0,
            popup_shadow_color: Color::TRANSPARENT,
            popup_shadow_blur: 0.0,
            popup_shadow_offset_y: 0.0,
        }
    }
}

pub fn current_theme_colors(theme: &Theme, is_dark: bool) -> ThemeColorTokens {
    ThemeColorTokens {
        background: theme.background,
        primary: theme.primary,
        surface: theme.surface,
        surface_hover: theme.surface_hover,
        surface_active: theme.surface_active,
        text: theme.text,
        border: theme.border,
        dark: is_dark,
    }
}

pub fn current_page_visuals(palette: &ThemeColorTokens) -> PageVisualTokens {
    PageVisualTokens {
        title_color: Color::new(palette.text.r, palette.text.g, palette.text.b, 0.98),
        subtitle_color: Color::new(palette.text.r, palette.text.g, palette.text.b, 0.72),
        body_color: Color::new(palette.text.r, palette.text.g, palette.text.b, 0.68),
        card_color: palette.surface,
        muted_card_color: palette.surface_hover,
        soft_accent_color: Color::new(palette.primary.r, palette.primary.g, palette.primary.b, 0.16),
        header_top_inset: 24.0,
        header_title_gap: 30.0,
        header_content_gap: 40.0,
        header_title_size: 31.0,
        header_subtitle_size: 20.0,
        section_gap: 16.0,
        section_inset: 20.0,
        section_rounding: 18.0,
        label_size: 17.0,
        field_height: 35.0,
    }
}

pub fn current_field_visuals(palette: &ThemeColorTokens) -> UIFieldVisualTokens {
    let mut tokens = UIFieldVisualTokens::default();
    if palette.dark {
        tokens.popup_shadow_color = Color::new(0.0, 0.0, 0.0, 0.28);
        tokens.popup_shadow_blur = 18.0;
        tokens.popup_shadow_offset_y = 8.0;
    } else {
        tokens.popup_shadow_color = Color::new(0.10, 0.14, 0.22, 0.14);
        tokens.popup_shadow_blur = 12.0;
        tokens.popup_shadow_offset_y = 5.0;
    }
    tokens
}
