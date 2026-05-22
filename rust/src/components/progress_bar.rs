use std::any::Any;

use crate::animation::Lerp;
// Note: Color will be used once renderer calls are implemented.
// use crate::color::Color;
use crate::rect::RectFrame;
use crate::state::UIState;
use crate::theme::{dark_theme, current_theme_colors};
use crate::ui::node::*;
use crate::ui::primitive::*;
use crate::renderer::renderer::Renderer;

/// Default width for a progress bar.
const DEFAULT_WIDTH: f32 = 300.0;
/// Default height for a progress bar.
const DEFAULT_HEIGHT: f32 = 15.0;
/// Speed factor for the value animation (units per second style lerp rate).
const ANIM_SPEED: f32 = 8.0;

/// A horizontal progress bar that fills from left to right.
pub struct ProgressBarNode {
    pub base: NodeBase,
    /// Target progress value in [0, 1].
    pub value: f32,
    /// The currently displayed (animated) value. Starts at -1.0 to signal
    /// that it has not yet been initialised and should snap to `value`.
    pub animated_value: f32,
}

impl ProgressBarNode {
    pub fn new(key: String) -> Self {
        let mut node = Self {
            base: NodeBase::new(key),
            value: 0.0,
            animated_value: -1.0,
        };
        node.base.primitive.width = DEFAULT_WIDTH;
        node.base.primitive.height = DEFAULT_HEIGHT;
        node
    }
}

impl UINode for ProgressBarNode {
    fn type_name(&self) -> &'static str {
        "ProgressBar"
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
        let target = self.value.clamp(0.0, 1.0);

        if self.animated_value < 0.0 {
            // First frame: snap to current value.
            self.animated_value = target;
        } else {
            // Smoothly animate towards the target value.
            let speed = (state.delta_time * ANIM_SPEED).clamp(0.0, 1.0);
            self.animated_value = self.animated_value.lerp(&target, speed);
        }

        // Keep repainting while the animation is in flight.
        if (self.animated_value - target).abs() > 0.001 {
            self.base.request_visual_repaint();
        }
    }

    fn draw(&self, renderer: &mut Renderer) {
        let frame = primitive_frame(&self.base.primitive);
        let opacity = self.base.primitive.opacity;
        let rounding = self.base.primitive.rounding.max(frame.height * 0.5);

        let theme = dark_theme();
        let palette = current_theme_colors(&theme, true);

        let track_color = apply_opacity(palette.surface_hover, opacity);
        renderer.draw_rect_simple(frame.x, frame.y, frame.width, frame.height, &track_color, rounding);

        let fill_width = frame.width * self.animated_value.clamp(0.0, 1.0);
        if fill_width > 0.0 {
            let fill_color = apply_opacity(palette.primary, opacity);
            renderer.draw_rect_simple(frame.x, frame.y, fill_width, frame.height, &fill_color, rounding);
        }
    }

    fn reset_defaults(&mut self) {
        self.value = 0.0;
        // Do not reset animated_value; it is runtime state.
        self.base.primitive.width = DEFAULT_WIDTH;
        self.base.primitive.height = DEFAULT_HEIGHT;
    }

    fn layout_bounds(&self) -> RectFrame {
        let p = &self.base.primitive;
        RectFrame {
            x: 0.0,
            y: 0.0,
            width: p.width,
            height: p.height,
        }
    }

    fn wants_continuous_update(&self) -> bool {
        let target = self.value.clamp(0.0, 1.0);
        (self.animated_value - target).abs() > 0.001
    }

    fn uses_cached_surface(&self) -> bool {
        true
    }

    fn begin_compose(&mut self, stamp: u64) {
        self.base.begin_compose(stamp);
    }

    fn finish_compose(&mut self) {
        self.base.track_compose_value("value", &self.value);
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
