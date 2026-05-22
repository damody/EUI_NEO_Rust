use std::any::Any;

use crate::color::Color;
use crate::rect::RectFrame;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::theme::{dark_theme, current_theme_colors};
use crate::ui::node::*;
use crate::ui::primitive::*;

/// A simple text label node.
pub struct LabelNode {
    pub base: NodeBase,
    pub text: String,
    pub font_size: f32,
    pub color: Color,
    pub use_theme_color: bool,
}

impl LabelNode {
    pub fn new(key: String) -> Self {
        Self {
            base: NodeBase::new(key),
            text: String::new(),
            font_size: 24.0,
            color: Color::WHITE,
            use_theme_color: true,
        }
    }

    /// Resolve the effective text color, applying theme if enabled.
    pub fn effective_color(&self) -> Color {
        if self.use_theme_color {
            // Use theme text color; fall back to dark theme defaults.
            let theme = dark_theme();
            let palette = current_theme_colors(&theme, true);
            apply_opacity(palette.text, self.base.primitive.opacity)
        } else {
            apply_opacity(self.color, self.base.primitive.opacity)
        }
    }

    /// Compute the text scale factor relative to the base size of 24.
    pub fn text_scale(&self) -> f32 {
        self.font_size / 24.0
    }
}

impl UINode for LabelNode {
    fn type_name(&self) -> &'static str {
        "Label"
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

    fn update(&mut self, _state: &UIState) {
        // Label has no interactive behaviour; nothing to update.
    }

    fn draw(&self, renderer: &mut Renderer) {
        let frame = primitive_frame(&self.base.primitive);
        let color = self.effective_color();
        let scale = self.text_scale();
        renderer.draw_text(&self.text, frame.x, frame.y, &color, scale);
    }

    fn reset_defaults(&mut self) {
        self.text.clear();
        self.font_size = 24.0;
        self.color = Color::WHITE;
        self.use_theme_color = true;
    }

    fn layout_bounds(&self) -> RectFrame {
        let _scale = self.text_scale();
        // TODO: let bounds = Renderer::MeasureTextBounds(&self.text, _scale);
        // For now return the primitive size as a placeholder.
        let p = &self.base.primitive;
        RectFrame {
            x: 0.0,
            y: 0.0,
            width: p.width,
            height: p.height,
        }
    }

    fn paint_bounds(&self) -> RectFrame {
        let frame = primitive_frame(&self.base.primitive);
        let layout = self.layout_bounds();
        // Clip layout bounds to the frame.
        let p = &self.base.primitive;
        if p.has_clip_rect {
            clip_frame(
                &RectFrame::new(
                    frame.x,
                    frame.y,
                    layout.width,
                    layout.height,
                ),
                &p.clip_rect,
            )
        } else {
            RectFrame::new(frame.x, frame.y, layout.width, layout.height)
        }
    }

    fn wants_continuous_update(&self) -> bool {
        false
    }

    fn uses_cached_surface(&self) -> bool {
        true
    }

    fn begin_compose(&mut self, stamp: u64) {
        self.base.begin_compose(stamp);
    }

    fn finish_compose(&mut self) {
        self.base.track_compose_value("text", &self.text);
        self.base.track_compose_value("font_size", &self.font_size);
        self.base.track_compose_value("use_theme_color", &self.use_theme_color);
        if !self.use_theme_color {
            self.base.track_compose_value("color_r", &self.color.r);
            self.base.track_compose_value("color_g", &self.color.g);
            self.base.track_compose_value("color_b", &self.color.b);
            self.base.track_compose_value("color_a", &self.color.a);
        }
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
