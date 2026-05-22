// ThemeTokens - translated from C++ ThemeTokens.h
//
// Re-exports the token structs already defined in crate::theme and adds
// helper functions that mirror the C++ inline helpers.

use crate::color::{Color, lerp_color};
use crate::rect::RectFrame;
use crate::theme::{
    ThemeColorTokens, PageVisualTokens, UIFieldVisualTokens,
    current_theme_colors, current_page_visuals, current_field_visuals,
    Theme,
};
use crate::types::{RectGradient, RectStyle};
use crate::ui::context::UIContext;
use crate::ui::primitive::{UIPrimitive, apply_opacity, make_style, primitive_frame};

// Re-export the token structs so callers can `use crate::theme_tokens::*`.
pub use crate::theme::{
    ThemeColorTokens as ThemeColorTokensAlias,
    PageVisualTokens as PageVisualTokensAlias,
    UIFieldVisualTokens as UIFieldVisualTokensAlias,
};

/// Layout measurements returned by `compose_page_header`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageHeaderLayout {
    pub title_y: f32,
    pub subtitle_y: f32,
    pub content_y: f32,
}

impl Default for PageHeaderLayout {
    fn default() -> Self {
        Self {
            title_y: 0.0,
            subtitle_y: 0.0,
            content_y: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Standalone theme color constructors (mirror C++ LightThemeColors / DarkThemeColors)
// ---------------------------------------------------------------------------

pub fn light_theme_colors() -> ThemeColorTokens {
    ThemeColorTokens {
        background: Color::new(0.95, 0.95, 0.97, 1.0),
        primary: Color::new(0.20, 0.50, 0.90, 1.0),
        surface: Color::new(1.00, 1.00, 1.00, 1.0),
        surface_hover: Color::new(0.90, 0.90, 0.90, 1.0),
        surface_active: Color::new(0.80, 0.80, 0.80, 1.0),
        text: Color::new(0.00, 0.00, 0.00, 1.0),
        border: Color::new(0.80, 0.80, 0.80, 1.0),
        dark: false,
    }
}

pub fn dark_theme_colors() -> ThemeColorTokens {
    ThemeColorTokens {
        background: Color::new(0.10, 0.10, 0.12, 1.0),
        primary: Color::new(0.30, 0.60, 1.00, 1.0),
        surface: Color::new(0.15, 0.15, 0.18, 1.0),
        surface_hover: Color::new(0.25, 0.25, 0.28, 1.0),
        surface_active: Color::new(0.35, 0.35, 0.38, 1.0),
        text: Color::new(1.00, 1.00, 1.00, 1.0),
        border: Color::new(0.30, 0.30, 0.30, 1.0),
        dark: true,
    }
}

/// Build a `Theme` struct from `ThemeColorTokens`.
pub fn make_theme(tokens: &ThemeColorTokens) -> Theme {
    Theme {
        background: tokens.background,
        primary: tokens.primary,
        surface: tokens.surface,
        surface_hover: tokens.surface_hover,
        surface_active: tokens.surface_active,
        text: tokens.text,
        border: tokens.border,
    }
}

// ---------------------------------------------------------------------------
// Field helpers
// ---------------------------------------------------------------------------

/// Resolve the fill color for a field primitive based on hover/active amounts.
pub fn resolve_field_fill(
    primitive: &UIPrimitive,
    palette: &ThemeColorTokens,
    hover_amount: f32,
    active_amount: f32,
) -> Color {
    let hover = hover_amount.clamp(0.0, 1.0);
    let active = active_amount.clamp(0.0, 1.0);
    let base_color = if primitive.background.a > 0.0 {
        primitive.background
    } else {
        palette.surface
    };
    let hover_color = if primitive.background.a > 0.0 {
        lerp_color(&base_color, &palette.surface_hover, 0.65)
    } else {
        palette.surface_hover
    };
    let active_blend = lerp_color(&base_color, &palette.surface_active, active);
    lerp_color(&active_blend, &hover_color, hover)
}

/// Draw the chrome (background + border + focus line) for a field.
pub fn draw_field_chrome(
    _primitive: &UIPrimitive,
    _palette: &ThemeColorTokens,
    _field_visuals: &UIFieldVisualTokens,
    _hover_amount: f32,
    _active_amount: f32,
    _rounding: f32,
) {
    // TODO: Renderer::draw_rect calls for background, border line, and focus accent line
}

/// Compute the popup list frame positioned below an anchor.
pub fn popup_list_frame(anchor_frame: &RectFrame, visible_height: f32, overlap: f32) -> RectFrame {
    RectFrame::new(
        anchor_frame.x,
        anchor_frame.y + anchor_frame.height - overlap,
        anchor_frame.width,
        visible_height + overlap,
    )
}

/// Build a `RectStyle` suitable for popup chrome.
pub fn make_popup_chrome_style(
    primitive: &UIPrimitive,
    palette: &ThemeColorTokens,
    field_visuals: &UIFieldVisualTokens,
    rounding: f32,
) -> RectStyle {
    let corner_radius = if rounding > 0.0 {
        rounding
    } else {
        field_visuals.popup_rounding
    };
    let mut style = make_style(primitive);
    style.color = apply_opacity(palette.surface, primitive.opacity);
    style.gradient = RectGradient::default();
    style.rounding = corner_radius;
    style.shadow_blur = field_visuals.popup_shadow_blur;
    style.shadow_offset_x = 0.0;
    style.shadow_offset_y = field_visuals.popup_shadow_offset_y;
    style.shadow_color = apply_opacity(field_visuals.popup_shadow_color, primitive.opacity);
    style
}

/// Draw popup chrome for a given frame.
pub fn draw_popup_chrome(
    _primitive: &UIPrimitive,
    _palette: &ThemeColorTokens,
    _field_visuals: &UIFieldVisualTokens,
    _frame: &RectFrame,
    _rounding: f32,
) {
    // TODO: Renderer::draw_rect(frame, make_popup_chrome_style(...))
}

/// Compose a page header (title + subtitle labels) and return layout measurements.
pub fn compose_page_header(
    _ui: &mut UIContext,
    _id_prefix: &str,
    bounds: &RectFrame,
    _title: &str,
    _subtitle: &str,
    visuals: &PageVisualTokens,
) -> PageHeaderLayout {
    let title_y = bounds.y + visuals.header_top_inset;
    let subtitle_y = title_y + visuals.header_title_gap;
    let content_y = subtitle_y + visuals.header_content_gap;

    // TODO: ui.label(id_prefix + ".title").text(title).position(...).font_size(...).color(...).build();
    // TODO: ui.label(id_prefix + ".subtitle").text(subtitle).position(...).font_size(...).color(...).build();

    PageHeaderLayout {
        title_y,
        subtitle_y,
        content_y,
    }
}

/// Compose a page section background panel.
pub fn compose_page_section(
    _ui: &mut UIContext,
    _id_prefix: &str,
    _bounds: &RectFrame,
    _background: Option<Color>,
    _visuals: &PageVisualTokens,
) {
    // TODO: ui.panel(id_prefix).position(...).size(...).background(...).rounding(...).build();
}
