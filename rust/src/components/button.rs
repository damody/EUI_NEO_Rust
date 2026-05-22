use std::any::Any;

use crate::animation::Lerp;
use crate::color::{Color, lerp_color};
// Note: Easing and RectFrame will be used once renderer calls are implemented.
// use crate::easing::Easing;
// use crate::rect::RectFrame;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::theme::{dark_theme, current_theme_colors};
use crate::ui::node::*;
use crate::ui::primitive::*;

/// Visual style of the button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Default,
    Primary,
    Outline,
}

impl std::default::Default for ButtonStyle {
    fn default() -> Self {
        ButtonStyle::Default
    }
}

/// Placement of an optional icon relative to the label text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonIconPlacement {
    Leading,
    Trailing,
}

impl std::default::Default for ButtonIconPlacement {
    fn default() -> Self {
        ButtonIconPlacement::Leading
    }
}

/// A clickable button node with hover/press animation.
pub struct ButtonNode {
    pub base: NodeBase,
    // Configuration
    pub text: String,
    pub icon: String,
    pub font_size: f32,
    pub style: ButtonStyle,
    pub icon_placement: ButtonIconPlacement,
    pub text_color_override: Option<Color>,
    pub hover_scale_idle: f32,
    pub hover_scale_hover: f32,
    pub hover_scale_duration: f32,
    // Callback
    pub on_click: Option<Box<dyn Fn()>>,
    // Runtime animation state
    pub hover_anim: f32,
    pub click_anim: f32,
}

impl ButtonNode {
    pub fn new(key: String) -> Self {
        Self {
            base: NodeBase::new(key),
            text: String::new(),
            icon: String::new(),
            font_size: 20.0,
            style: ButtonStyle::Default,
            icon_placement: ButtonIconPlacement::Leading,
            text_color_override: None,
            hover_scale_idle: 1.0,
            hover_scale_hover: 1.0,
            hover_scale_duration: 0.16,
            on_click: None,
            hover_anim: 0.0,
            click_anim: 0.0,
        }
    }

    /// Resolve the fill color for the current style and hover/click blend.
    fn fill_color(&self) -> Color {
        let theme = dark_theme();
        let palette = current_theme_colors(&theme, true);
        let opacity = self.base.primitive.opacity;

        match self.style {
            ButtonStyle::Primary => {
                let base = palette.primary;
                let hover = Color::new(
                    (base.r + 0.1).min(1.0),
                    (base.g + 0.1).min(1.0),
                    (base.b + 0.1).min(1.0),
                    base.a,
                );
                let blended = lerp_color(&base, &hover, self.hover_anim);
                apply_opacity(blended, opacity)
            }
            ButtonStyle::Default => {
                let idle = palette.surface;
                let hover = palette.surface_hover;
                let blended = lerp_color(&idle, &hover, self.hover_anim);
                apply_opacity(blended, opacity)
            }
            ButtonStyle::Outline => {
                let idle = Color::TRANSPARENT;
                let hover = Color::new(
                    palette.surface_hover.r,
                    palette.surface_hover.g,
                    palette.surface_hover.b,
                    0.5,
                );
                let blended = lerp_color(&idle, &hover, self.hover_anim);
                apply_opacity(blended, opacity)
            }
        }
    }

    /// Resolve the text color for the current style.
    fn text_color(&self) -> Color {
        if let Some(color) = self.text_color_override {
            return apply_opacity(color, self.base.primitive.opacity);
        }

        let theme = dark_theme();
        let palette = current_theme_colors(&theme, true);
        let opacity = self.base.primitive.opacity;

        match self.style {
            ButtonStyle::Primary => apply_opacity(Color::WHITE, opacity),
            _ => apply_opacity(palette.text, opacity),
        }
    }

    /// Resolve the border color (only relevant for Outline style).
    fn border_color(&self) -> Color {
        let theme = dark_theme();
        let palette = current_theme_colors(&theme, true);
        let opacity = self.base.primitive.opacity;
        match self.style {
            ButtonStyle::Outline => apply_opacity(palette.border, opacity),
            _ => Color::TRANSPARENT,
        }
    }

    /// Compute the current scale based on hover animation.
    fn current_scale(&self) -> f32 {
        self.hover_scale_idle.lerp(&self.hover_scale_hover, self.hover_anim)
    }

    fn text_scale(&self) -> f32 {
        self.font_size / 24.0
    }
}

impl UINode for ButtonNode {
    fn type_name(&self) -> &'static str {
        "Button"
    }

    fn key(&self) -> &str {
        &self.base.key
    }

    fn primitive(&self) -> &UIPrimitive {
        &self.base.primitive
    }

    fn primitive_mut(&mut self) -> &mut UIPrimitive {
        &mut self.base.primitive
    }

    fn update(&mut self, state: &UIState) {
        let frame = primitive_frame(&self.base.primitive);
        let hovered = frame.contains(state.mouse_x, state.mouse_y)
            && self.base.primitive.visible
            && self.base.primitive.enabled;

        let dt = state.delta_time;

        // Hover animation: lerp towards 1.0 when hovered, 0.0 otherwise.
        let target_hover: f32 = if hovered { 1.0 } else { 0.0 };
        if self.hover_scale_duration > 0.0 {
            let speed = dt / self.hover_scale_duration;
            self.hover_anim = self.hover_anim.lerp(&target_hover, speed.clamp(0.0, 1.0));
        } else {
            self.hover_anim = target_hover;
        }

        // Click animation: decay towards 0.
        if self.click_anim > 0.0 {
            self.click_anim = (self.click_anim - dt * 4.0).max(0.0);
        }

        // Click detection.
        if hovered && state.mouse_clicked {
            self.click_anim = 1.0;
            if let Some(ref cb) = self.on_click {
                cb();
            }
        }

        // Mark dirty when animating.
        if self.hover_anim > 0.001 && self.hover_anim < 0.999 || self.click_anim > 0.001 {
            self.base.request_visual_repaint();
        }
    }

    fn draw(&self, renderer: &mut Renderer) {
        let frame = primitive_frame(&self.base.primitive);
        let fill = self.fill_color();
        let text_col = self.text_color();
        let border = self.border_color();
        let scale = self.current_scale();
        let rounding = self.base.primitive.rounding;
        let text_scale = self.text_scale();

        // Apply scale transform around center.
        let cx = frame.x + frame.width * 0.5;
        let cy = frame.y + frame.height * 0.5;
        let sx = frame.width * scale;
        let sy = frame.height * scale;
        let draw_x = cx - sx * 0.5;
        let draw_y = cy - sy * 0.5;

        renderer.draw_rect_simple(draw_x, draw_y, sx, sy, &fill, rounding);

        // Border for outline style.
        if self.style == ButtonStyle::Outline {
            let bw = 1.0;
            renderer.draw_rect_simple(draw_x - bw, draw_y - bw, sx + bw * 2.0, sy + bw * 2.0, &border, rounding + bw);
        }

        // Click flash overlay.
        if self.click_anim > 0.0 {
            let flash = Color::new(1.0, 1.0, 1.0, 0.15 * self.click_anim);
            renderer.draw_rect_simple(draw_x, draw_y, sx, sy, &flash, rounding);
        }

        // Layout icon and text inside the button.
        let icon_gap: f32 = if !self.icon.is_empty() && !self.text.is_empty() { 6.0 } else { 0.0 };

        let text_w = renderer.measure_text_width(&self.text, text_scale);
        let icon_w: f32 = if !self.icon.is_empty() { renderer.measure_text_width(&self.icon, text_scale) } else { 0.0 };

        let total_w = icon_w + icon_gap + text_w;
        let start_x = draw_x + (sx - total_w) * 0.5;
        let text_y = draw_y + (sy - self.font_size) * 0.5;

        match self.icon_placement {
            ButtonIconPlacement::Leading => {
                if !self.icon.is_empty() {
                    renderer.draw_text(&self.icon, start_x, text_y, &text_col, text_scale);
                }
                renderer.draw_text(&self.text, start_x + icon_w + icon_gap, text_y, &text_col, text_scale);
            }
            ButtonIconPlacement::Trailing => {
                renderer.draw_text(&self.text, start_x, text_y, &text_col, text_scale);
                if !self.icon.is_empty() {
                    renderer.draw_text(&self.icon, start_x + text_w + icon_gap, text_y, &text_col, text_scale);
                }
            }
        }
    }

    fn reset_defaults(&mut self) {
        self.text.clear();
        self.icon.clear();
        self.font_size = 20.0;
        self.style = ButtonStyle::Default;
        self.icon_placement = ButtonIconPlacement::Leading;
        self.text_color_override = None;
        self.hover_scale_idle = 1.0;
        self.hover_scale_hover = 1.0;
        self.hover_scale_duration = 0.16;
        self.on_click = None;
        // Note: hover_anim and click_anim are runtime state, not reset.
    }

    fn wants_continuous_update(&self) -> bool {
        self.hover_anim > 0.001 || self.click_anim > 0.001
    }

    fn uses_cached_surface(&self) -> bool {
        true
    }

    fn begin_compose(&mut self, stamp: u64) {
        self.base.begin_compose(stamp);
    }

    fn finish_compose(&mut self) {
        self.base.track_compose_value("text", &self.text);
        self.base.track_compose_value("icon", &self.icon);
        self.base.track_compose_value("font_size", &self.font_size);
        self.base.track_compose_value("style", &(self.style as u32));
        self.base.track_compose_value("icon_placement", &(self.icon_placement as u32));
        self.base.track_compose_value("text_color_override", &self.text_color_override.is_some());
        if let Some(color) = self.text_color_override {
            self.base.track_compose_value("text_color_r", &color.r);
            self.base.track_compose_value("text_color_g", &color.g);
            self.base.track_compose_value("text_color_b", &color.b);
            self.base.track_compose_value("text_color_a", &color.a);
        }
        self.base.track_compose_value("hover_scale_idle", &self.hover_scale_idle);
        self.base.track_compose_value("hover_scale_hover", &self.hover_scale_hover);
        self.base.finish_compose();
    }

    fn composed_in(&self, stamp: u64) -> bool {
        self.base.composed_in(stamp)
    }

    fn cache_dirty(&self) -> bool {
        self.base.cache_dirty
    }

    fn clear_cache_dirty(&mut self) {
        self.base.cache_dirty = false;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_color_uses_override_with_primitive_opacity() {
        let mut button = ButtonNode::new("button".to_string());
        button.text_color_override = Some(Color::new(0.2, 0.4, 0.6, 0.8));
        button.base.primitive.opacity = 0.5;

        assert_eq!(button.text_color(), Color::new(0.2, 0.4, 0.6, 0.4));
    }

    #[test]
    fn reset_defaults_clears_text_color_override() {
        let mut button = ButtonNode::new("button".to_string());
        button.text_color_override = Some(Color::new(0.2, 0.4, 0.6, 0.8));

        button.reset_defaults();

        assert_eq!(button.text_color_override, None);
    }
}
