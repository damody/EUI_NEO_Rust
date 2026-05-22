use crate::color::{Color, lerp_color};
use crate::rect::RectFrame;

/// Transform applied to a rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectTransform {
    pub translate_x: f32,
    pub translate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_degrees: f32,
}

impl Default for RectTransform {
    fn default() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_degrees: 0.0,
        }
    }
}

pub fn lerp_transform(a: &RectTransform, b: &RectTransform, t: f32) -> RectTransform {
    let t = t.clamp(0.0, 1.0);
    RectTransform {
        translate_x: a.translate_x + (b.translate_x - a.translate_x) * t,
        translate_y: a.translate_y + (b.translate_y - a.translate_y) * t,
        scale_x: a.scale_x + (b.scale_x - a.scale_x) * t,
        scale_y: a.scale_y + (b.scale_y - a.scale_y) * t,
        rotation_degrees: a.rotation_degrees + (b.rotation_degrees - a.rotation_degrees) * t,
    }
}

/// Four-corner gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectGradient {
    pub enabled: bool,
    pub top_left: Color,
    pub top_right: Color,
    pub bottom_left: Color,
    pub bottom_right: Color,
}

impl Default for RectGradient {
    fn default() -> Self {
        Self {
            enabled: false,
            top_left: Color::WHITE,
            top_right: Color::WHITE,
            bottom_left: Color::WHITE,
            bottom_right: Color::WHITE,
        }
    }
}

impl RectGradient {
    pub fn solid(color: Color) -> Self {
        Self { enabled: true, top_left: color, top_right: color, bottom_left: color, bottom_right: color }
    }

    pub fn horizontal(left: Color, right: Color) -> Self {
        Self { enabled: true, top_left: left, top_right: right, bottom_left: left, bottom_right: right }
    }

    pub fn vertical(top: Color, bottom: Color) -> Self {
        Self { enabled: true, top_left: top, top_right: top, bottom_left: bottom, bottom_right: bottom }
    }

    pub fn corners(top_left: Color, top_right: Color, bottom_left: Color, bottom_right: Color) -> Self {
        Self { enabled: true, top_left, top_right, bottom_left, bottom_right }
    }
}

pub fn lerp_gradient(a: &RectGradient, b: &RectGradient, t: f32) -> RectGradient {
    let t = t.clamp(0.0, 1.0);
    RectGradient {
        enabled: a.enabled || b.enabled,
        top_left: lerp_color(&a.top_left, &b.top_left, t),
        top_right: lerp_color(&a.top_right, &b.top_right, t),
        bottom_left: lerp_color(&a.bottom_left, &b.bottom_left, t),
        bottom_right: lerp_color(&a.bottom_right, &b.bottom_right, t),
    }
}

/// Shadow parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIShadow {
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub color: Color,
}

impl Default for UIShadow {
    fn default() -> Self {
        Self { blur: 0.0, offset_x: 0.0, offset_y: 0.0, color: Color::TRANSPARENT }
    }
}

/// Style bundle for rect drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectStyle {
    pub color: Color,
    pub gradient: RectGradient,
    pub rounding: f32,
    pub blur_amount: f32,
    pub shadow_blur: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub shadow_color: Color,
    pub transform: RectTransform,
}

impl Default for RectStyle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            gradient: RectGradient::default(),
            rounding: 0.0,
            blur_amount: 0.0,
            shadow_blur: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_color: Color::TRANSPARENT,
            transform: RectTransform::default(),
        }
    }
}

pub fn lerp_rect_style(a: &RectStyle, b: &RectStyle, t: f32) -> RectStyle {
    let t = t.clamp(0.0, 1.0);
    let lerp_f = |a: f32, b: f32| a + (b - a) * t;
    RectStyle {
        color: lerp_color(&a.color, &b.color, t),
        gradient: lerp_gradient(&a.gradient, &b.gradient, t),
        rounding: lerp_f(a.rounding, b.rounding),
        blur_amount: lerp_f(a.blur_amount, b.blur_amount),
        shadow_blur: lerp_f(a.shadow_blur, b.shadow_blur),
        shadow_offset_x: lerp_f(a.shadow_offset_x, b.shadow_offset_x),
        shadow_offset_y: lerp_f(a.shadow_offset_y, b.shadow_offset_y),
        shadow_color: lerp_color(&a.shadow_color, &b.shadow_color, t),
        transform: lerp_transform(&a.transform, &b.transform, t),
    }
}

/// Panel visual state (for animation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelState {
    pub frame: RectFrame,
    pub style: RectStyle,
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            frame: RectFrame::ZERO,
            style: RectStyle::default(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// Render layer ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RenderLayer {
    Backdrop = 0,
    Content = 1,
    Chrome = 2,
    Popup = 3,
}

impl Default for RenderLayer {
    fn default() -> Self { Self::Content }
}

/// Anchor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    TopLeft, TopCenter, TopRight,
    CenterLeft, Center, CenterRight,
    BottomLeft, BottomCenter, BottomRight,
}

impl Default for Anchor {
    fn default() -> Self { Self::TopLeft }
}
