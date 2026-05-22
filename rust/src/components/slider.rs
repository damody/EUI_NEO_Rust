use std::any::Any;

use crate::animation::Lerp;
use crate::color::{Color, lerp_color};
use crate::rect::RectFrame;
use crate::state::UIState;
use crate::theme::{dark_theme, current_theme_colors};
use crate::ui::node::*;
use crate::ui::primitive::*;
use crate::renderer::renderer::Renderer;

/// Default width for a slider track.
const DEFAULT_WIDTH: f32 = 300.0;
/// Default height for the slider widget (includes handle overshoot).
const DEFAULT_HEIGHT: f32 = 20.0;
/// Track height (the thin rail).
const TRACK_HEIGHT: f32 = 4.0;
/// Base handle radius.
const HANDLE_RADIUS_BASE: f32 = 7.0;
/// Expanded handle radius on hover/active.
const HANDLE_RADIUS_ACTIVE: f32 = 10.0;
/// Animation speed for hover/active transitions.
const ANIM_SPEED: f32 = 10.0;

/// A horizontal slider that reports a value in [0, 1].
pub struct SliderNode {
    pub base: NodeBase,
    /// Current slider value in [0, 1].
    pub value: f32,
    /// Callback invoked when the value changes during a drag.
    pub on_change: Option<Box<dyn Fn(f32)>>,
    /// Whether the user is currently dragging the handle.
    pub is_dragging: bool,
    /// Hover animation factor [0, 1].
    pub hover_anim: f32,
    /// Active (pressed) animation factor [0, 1].
    pub active_anim: f32,
}

impl SliderNode {
    pub fn new(key: String) -> Self {
        let mut node = Self {
            base: NodeBase::new(key),
            value: 0.0,
            on_change: None,
            is_dragging: false,
            hover_anim: 0.0,
            active_anim: 0.0,
        };
        node.base.primitive.width = DEFAULT_WIDTH;
        node.base.primitive.height = DEFAULT_HEIGHT;
        node
    }

    /// Compute the X position of the handle center for the current value.
    fn handle_x(&self, frame: &RectFrame) -> f32 {
        let usable = frame.width - HANDLE_RADIUS_ACTIVE * 2.0;
        frame.x + HANDLE_RADIUS_ACTIVE + usable * self.value.clamp(0.0, 1.0)
    }

    /// Convert a screen-space X coordinate to a slider value in [0, 1].
    fn x_to_value(&self, frame: &RectFrame, screen_x: f32) -> f32 {
        let usable = frame.width - HANDLE_RADIUS_ACTIVE * 2.0;
        if usable <= 0.0 {
            return 0.0;
        }
        ((screen_x - frame.x - HANDLE_RADIUS_ACTIVE) / usable).clamp(0.0, 1.0)
    }

    /// Current handle radius based on emphasis (hover + active).
    fn handle_radius(&self) -> f32 {
        let emphasis = self.hover_anim.max(self.active_anim);
        HANDLE_RADIUS_BASE.lerp(&HANDLE_RADIUS_ACTIVE, emphasis)
    }
}

impl UINode for SliderNode {
    fn type_name(&self) -> &'static str {
        "Slider"
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
        let dt = state.delta_time;

        // Hit-test against the full widget area.
        let hovered = frame.contains(state.mouse_x, state.mouse_y)
            && self.base.primitive.visible
            && self.base.primitive.enabled;

        // --- Drag handling ---
        if self.is_dragging {
            if state.mouse_down {
                let new_val = self.x_to_value(&frame, state.mouse_x);
                if (new_val - self.value).abs() > 0.0001 {
                    self.value = new_val;
                    if let Some(ref cb) = self.on_change {
                        cb(self.value);
                    }
                    self.base.request_visual_repaint();
                }
            } else {
                // Mouse released: stop dragging.
                self.is_dragging = false;
            }
        } else if hovered && state.mouse_clicked {
            self.is_dragging = true;
            let new_val = self.x_to_value(&frame, state.mouse_x);
            self.value = new_val;
            if let Some(ref cb) = self.on_change {
                cb(self.value);
            }
            self.base.request_visual_repaint();
        }

        // --- Hover animation ---
        let target_hover: f32 = if hovered || self.is_dragging { 1.0 } else { 0.0 };
        let speed_hover = (dt * ANIM_SPEED).clamp(0.0, 1.0);
        self.hover_anim = self.hover_anim.lerp(&target_hover, speed_hover);

        // --- Active animation ---
        let target_active: f32 = if self.is_dragging { 1.0 } else { 0.0 };
        let speed_active = (dt * ANIM_SPEED).clamp(0.0, 1.0);
        self.active_anim = self.active_anim.lerp(&target_active, speed_active);

        // Repaint while animating.
        if (self.hover_anim - target_hover).abs() > 0.001
            || (self.active_anim - target_active).abs() > 0.001
        {
            self.base.request_visual_repaint();
        }
    }

    fn draw(&self, renderer: &mut Renderer) {
        let frame = primitive_frame(&self.base.primitive);
        let opacity = self.base.primitive.opacity;

        let theme = dark_theme();
        let palette = current_theme_colors(&theme, true);

        let track_y = frame.y + (frame.height - TRACK_HEIGHT) * 0.5;
        let track_rounding = TRACK_HEIGHT * 0.5;

        let track_bg = apply_opacity(palette.surface_hover, opacity);
        renderer.draw_rect_simple(frame.x, track_y, frame.width, TRACK_HEIGHT, &track_bg, track_rounding);

        let hx = self.handle_x(&frame);
        let fill_width = hx - frame.x;
        if fill_width > 0.0 {
            let fill_color = apply_opacity(palette.primary, opacity);
            renderer.draw_rect_simple(frame.x, track_y, fill_width, TRACK_HEIGHT, &fill_color, track_rounding);
        }

        let radius = self.handle_radius();
        let handle_cy = frame.y + frame.height * 0.5;
        let emphasis = self.hover_anim.max(self.active_anim);
        let handle_base = palette.primary;
        let handle_light = Color::new(
            (handle_base.r + 0.15).min(1.0),
            (handle_base.g + 0.15).min(1.0),
            (handle_base.b + 0.15).min(1.0),
            handle_base.a,
        );
        let handle_color = apply_opacity(lerp_color(&handle_base, &handle_light, emphasis), opacity);
        let handle_x = hx - radius;
        let handle_y = handle_cy - radius;
        let handle_size = radius * 2.0;
        renderer.draw_rect_simple(handle_x, handle_y, handle_size, handle_size, &handle_color, radius);
    }

    fn reset_defaults(&mut self) {
        self.value = 0.0;
        self.on_change = None;
        self.base.primitive.width = DEFAULT_WIDTH;
        self.base.primitive.height = DEFAULT_HEIGHT;
        // Runtime animation state is preserved across resets.
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
        self.is_dragging || self.hover_anim > 0.001 || self.active_anim > 0.001
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
