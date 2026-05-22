use std::any::Any;

use crate::color::{Color, lerp_color};
use crate::rect::RectFrame;
use crate::state::UIState;
use crate::ui::node::{NodeBase, UINode};
use crate::ui::primitive::{UIPrimitive, primitive_frame};
use crate::renderer::renderer::Renderer;

/// Inline f32 lerp helper.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// ScrollAreaNode -- scrollable content region with a thumb bar.
pub struct ScrollAreaNode {
    pub base: NodeBase,

    // --- configuration ---
    pub content_height: f32,
    pub scroll_step: f32,
    pub scroll_offset_y: f32,

    // --- runtime ---
    pub hovered_: bool,
    pub is_dragging: bool,
    pub drag_grab_offset_y: f32,
    pub accent_anim: f32,
    pub accent_pulse: f32,
}

impl ScrollAreaNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        base.primitive.width = 200.0;
        base.primitive.height = 200.0;
        base.primitive.z_index = 48;
        Self {
            base,
            content_height: 0.0,
            scroll_step: 48.0,
            scroll_offset_y: 0.0,
            hovered_: false,
            is_dragging: false,
            drag_grab_offset_y: 0.0,
            accent_anim: 0.0,
            accent_pulse: 0.0,
        }
    }

    // --- public getters ---

    pub fn scroll_offset_y(&self) -> f32 {
        self.scroll_offset_y
    }

    // --- helpers ---

    pub fn max_scroll_offset(&self) -> f32 {
        let p = &self.base.primitive;
        (self.content_height - p.height).max(0.0)
    }

    pub fn clamp_scroll_offset(&mut self) {
        let max = self.max_scroll_offset();
        self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max);
    }

    /// Compute the thumb rectangle in screen space.
    pub fn thumb_frame(&self) -> RectFrame {
        let p = &self.base.primitive;
        let frame = primitive_frame(p);
        let visible_h = p.height;
        let total_h = self.content_height.max(visible_h);

        let thumb_bar_width = 6.0;
        let thumb_bar_inset = 3.0;

        let thumb_height = (visible_h / total_h * visible_h).max(24.0);
        let track_height = visible_h - thumb_height;
        let max_offset = self.max_scroll_offset();
        let ratio = if max_offset > 0.0 {
            self.scroll_offset_y / max_offset
        } else {
            0.0
        };
        let thumb_y = frame.y + ratio * track_height;
        let thumb_x = frame.x + frame.width - thumb_bar_width - thumb_bar_inset;

        RectFrame::new(thumb_x, thumb_y, thumb_bar_width, thumb_height)
    }

    /// Whether content is scrollable (content taller than viewport).
    fn is_scrollable(&self) -> bool {
        self.content_height > self.base.primitive.height
    }
}

impl UINode for ScrollAreaNode {
    fn type_name(&self) -> &'static str {
        "ScrollArea"
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
        let dt = state.delta_time;
        let frame = primitive_frame(&self.base.primitive);

        // Hover detection
        let hovered = frame.contains(state.mouse_x, state.mouse_y);
        self.hovered_ = hovered;

        if !self.is_scrollable() {
            self.scroll_offset_y = 0.0;
            self.accent_anim = lerp_f32(self.accent_anim, 0.0, dt * 8.0);
            self.accent_pulse = lerp_f32(self.accent_pulse, 0.0, dt * 4.0);
            return;
        }

        // Thumb dragging
        let thumb = self.thumb_frame();
        if state.mouse_clicked && thumb.contains(state.mouse_x, state.mouse_y) {
            self.is_dragging = true;
            self.drag_grab_offset_y = state.mouse_y - thumb.y;
        }
        if !state.mouse_down {
            self.is_dragging = false;
        }

        if self.is_dragging {
            let visible_h = self.base.primitive.height;
            let total_h = self.content_height.max(visible_h);
            let thumb_height = (visible_h / total_h * visible_h).max(24.0);
            let track_height = visible_h - thumb_height;

            if track_height > 0.0 {
                let new_thumb_y = state.mouse_y - self.drag_grab_offset_y - frame.y;
                let ratio = (new_thumb_y / track_height).clamp(0.0, 1.0);
                self.scroll_offset_y = ratio * self.max_scroll_offset();
            }
        }

        // Scroll wheel handling
        if hovered && !state.scroll_consumed && state.scroll_delta_y.abs() > 0.0 {
            self.scroll_offset_y -= state.scroll_delta_y * self.scroll_step;
            self.clamp_scroll_offset();
            self.accent_pulse = 1.0;
        }

        self.clamp_scroll_offset();

        // Accent animation (fade in when hovered or dragging)
        let target_accent = if hovered || self.is_dragging { 1.0 } else { 0.0 };
        self.accent_anim = lerp_f32(self.accent_anim, target_accent, dt * 8.0);
        self.accent_pulse = lerp_f32(self.accent_pulse, 0.0, dt * 4.0);

        self.base.request_visual_repaint();
    }

    fn draw(&self, renderer: &mut Renderer) {
        if !self.is_scrollable() {
            return;
        }
        let thumb = self.thumb_frame();
        let accent_t = self.accent_anim.clamp(0.0, 1.0);
        let theme = crate::theme::dark_theme();
        let idle_color = Color::new(0.5, 0.5, 0.5, 0.3);
        let accent_color = theme.primary.with_alpha(0.7 + 0.3 * self.accent_pulse);
        let thumb_color = lerp_color(&idle_color, &accent_color, accent_t);
        renderer.draw_rect_simple(thumb.x, thumb.y, thumb.width, thumb.height, &thumb_color, 3.0);
    }

    fn reset_defaults(&mut self) {
        self.base.primitive.width = 200.0;
        self.base.primitive.height = 200.0;
        self.base.primitive.z_index = 48;
    }

    fn wants_continuous_update(&self) -> bool {
        self.is_dragging || self.accent_anim > 0.01
    }

    fn begin_compose(&mut self, stamp: u64) {
        self.base.begin_compose(stamp);
    }

    fn finish_compose(&mut self) {
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
