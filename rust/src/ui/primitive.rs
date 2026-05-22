use crate::color::Color;
use crate::types::{RectGradient, RectTransform, RectStyle, UIShadow, RenderLayer, Anchor};
use crate::rect::RectFrame;

/// Clip rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UIClipRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Core primitive that every UINode wraps.
#[derive(Debug, Clone, PartialEq)]
pub struct UIPrimitive {
    pub x: f32,
    pub y: f32,
    pub context_offset_x: f32,
    pub context_offset_y: f32,
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub translate_x: f32,
    pub translate_y: f32,
    pub anchor: Anchor,
    pub rounding: f32,
    pub background: Color,
    pub gradient: RectGradient,
    pub border_width: f32,
    pub border_color: Color,
    pub blur: f32,
    pub shadow: UIShadow,
    pub opacity: f32,
    pub visible: bool,
    pub enabled: bool,
    pub render_layer: RenderLayer,
    pub z_index: i32,
    pub clip_to_parent: bool,
    pub has_clip_rect: bool,
    pub clip_rect: UIClipRect,
}

impl Default for UIPrimitive {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            context_offset_x: 0.0,
            context_offset_y: 0.0,
            width: 0.0,
            height: 0.0,
            min_width: 0.0,
            min_height: 0.0,
            max_width: 0.0,
            max_height: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
            anchor: Anchor::TopLeft,
            rounding: 0.0,
            background: Color::TRANSPARENT,
            gradient: RectGradient::default(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            blur: 0.0,
            shadow: UIShadow::default(),
            opacity: 1.0,
            visible: true,
            enabled: true,
            render_layer: RenderLayer::Content,
            z_index: 0,
            clip_to_parent: true,
            has_clip_rect: false,
            clip_rect: UIClipRect::default(),
        }
    }
}

/// Apply opacity to a color's alpha channel.
pub fn apply_opacity(color: Color, opacity: f32) -> Color {
    Color {
        a: color.a * opacity.clamp(0.0, 1.0),
        ..color
    }
}

/// Build a RectTransform from the primitive's transform fields.
pub fn make_transform(p: &UIPrimitive) -> RectTransform {
    RectTransform {
        translate_x: p.translate_x,
        translate_y: p.translate_y,
        scale_x: p.scale_x,
        scale_y: p.scale_y,
        rotation_degrees: p.rotation,
    }
}

/// Build a RectStyle from a primitive, applying opacity to all color fields.
pub fn make_style(p: &UIPrimitive) -> RectStyle {
    let effective_bg = apply_opacity(p.background, p.opacity);
    let effective_gradient = if p.gradient.enabled {
        let op = p.opacity;
        RectGradient {
            enabled: true,
            top_left: apply_opacity(p.gradient.top_left, op),
            top_right: apply_opacity(p.gradient.top_right, op),
            bottom_left: apply_opacity(p.gradient.bottom_left, op),
            bottom_right: apply_opacity(p.gradient.bottom_right, op),
        }
    } else {
        p.gradient
    };
    RectStyle {
        color: effective_bg,
        gradient: effective_gradient,
        rounding: p.rounding,
        blur_amount: p.blur,
        shadow_blur: p.shadow.blur,
        shadow_offset_x: p.shadow.offset_x,
        shadow_offset_y: p.shadow.offset_y,
        shadow_color: apply_opacity(p.shadow.color, p.opacity),
        transform: make_transform(p),
    }
}

/// Compute the on-screen frame of a primitive (position + context offset).
pub fn primitive_frame(p: &UIPrimitive) -> RectFrame {
    RectFrame {
        x: p.x + p.context_offset_x,
        y: p.y + p.context_offset_y,
        width: p.width,
        height: p.height,
    }
}

/// Hit-test: does the point (x, y) lie inside the primitive's frame?
pub fn primitive_contains(p: &UIPrimitive, x: f32, y: f32) -> bool {
    let frame = primitive_frame(p);
    x >= frame.x
        && x <= frame.x + frame.width
        && y >= frame.y
        && y <= frame.y + frame.height
}

/// Clip one RectFrame to a UIClipRect, returning the intersection.
pub fn clip_frame(frame: &RectFrame, clip: &UIClipRect) -> RectFrame {
    let x1 = frame.x.max(clip.x);
    let y1 = frame.y.max(clip.y);
    let x2 = (frame.x + frame.width).min(clip.x + clip.width);
    let y2 = (frame.y + frame.height).min(clip.y + clip.height);
    RectFrame {
        x: x1,
        y: y1,
        width: (x2 - x1).max(0.0),
        height: (y2 - y1).max(0.0),
    }
}
