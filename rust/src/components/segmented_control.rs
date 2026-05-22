use std::any::Any;

use crate::color::{Color, lerp_color};
use crate::rect::RectFrame;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::theme;
use crate::ui::node::{NodeBase, UINode};
use crate::ui::primitive::{UIPrimitive, primitive_frame};

/// Inline f32 lerp helper.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Evaluate a cubic Bezier curve at parameter t.
/// P0 = 0,0  P1 = (x1,y1)  P2 = (x2,y2)  P3 = 1,1
fn cubic_bezier_at(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    // We need to find the y value for a given t along the curve.
    // For a standard CSS-style cubic bezier, we solve for the x parameter
    // then sample y. For simplicity, we use the parametric form directly.
    let t = t.clamp(0.0, 1.0);

    // Parametric x(t) = 3*u^2*t*x1 + 3*u*t^2*x2 + t^3
    // We need to invert x(t) to find t for a given x.
    // Use Newton's method to find the t that gives us our input t as x.
    let mut guess = t;
    for _ in 0..8 {
        let u_g = 1.0 - guess;
        let x_val = 3.0 * u_g * u_g * guess * x1 + 3.0 * u_g * guess * guess * x2 + guess * guess * guess;
        let dx = 3.0 * u_g * u_g * x1 + 6.0 * u_g * guess * (x2 - x1) + 3.0 * guess * guess * (1.0 - x2);
        if dx.abs() < 1e-7 {
            break;
        }
        guess -= (x_val - t) / dx;
        guess = guess.clamp(0.0, 1.0);
    }

    let u_g = 1.0 - guess;
    3.0 * u_g * u_g * guess * y1 + 3.0 * u_g * guess * guess * y2 + guess * guess * guess
}

/// Standard ease-in-out Bezier (0.42, 0, 0.58, 1).
fn ease_in_out_bezier(t: f32) -> f32 {
    cubic_bezier_at(0.42, 0.0, 0.58, 1.0, t)
}

/// SegmentedControlNode -- horizontal segmented selector with animated indicator.
pub struct SegmentedControlNode {
    pub base: NodeBase,

    // --- configuration ---
    pub items: Vec<String>,
    pub selected_index: i32,
    pub font_size: f32,
    pub on_change: Option<Box<dyn Fn(i32, &str)>>,

    // --- animation state ---
    pub indicator_anim: f32,
    pub indicator_start: f32,
    pub indicator_target: f32,
    pub indicator_progress: f32,
    pub indicator_duration: f32,
    pub indicator_ready: bool,
    pub last_item_count: usize,
}

impl SegmentedControlNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        base.primitive.width = 300.0;
        base.primitive.height = 35.0;
        Self {
            base,
            items: Vec::new(),
            selected_index: 0,
            font_size: 20.0,
            on_change: None,
            indicator_anim: 0.0,
            indicator_start: 0.0,
            indicator_target: 0.0,
            indicator_progress: 1.0,
            indicator_duration: 0.22,
            indicator_ready: false,
            last_item_count: 0,
        }
    }

    /// Compute the x position of a segment by index.
    fn segment_x(&self, index: i32) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let frame = primitive_frame(&self.base.primitive);
        let segment_w = frame.width / self.items.len() as f32;
        frame.x + index as f32 * segment_w
    }

    /// Compute the width of each segment.
    fn segment_width(&self) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let frame = primitive_frame(&self.base.primitive);
        frame.width / self.items.len() as f32
    }
}

impl UINode for SegmentedControlNode {
    fn type_name(&self) -> &'static str {
        "SegmentedControl"
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

        // Reset indicator when items change
        if self.items.len() != self.last_item_count {
            self.last_item_count = self.items.len();
            self.indicator_ready = false;
        }

        // Initialize indicator position
        if !self.indicator_ready && !self.items.is_empty() {
            self.indicator_anim = self.selected_index as f32;
            self.indicator_start = self.indicator_anim;
            self.indicator_target = self.indicator_anim;
            self.indicator_progress = 1.0;
            self.indicator_ready = true;
        }

        // Click detection
        if state.mouse_clicked && frame.contains(state.mouse_x, state.mouse_y) && !self.items.is_empty() {
            let segment_w = self.segment_width();
            let relative_x = state.mouse_x - frame.x;
            let clicked_index = (relative_x / segment_w) as i32;
            let clicked_index = clicked_index.clamp(0, self.items.len() as i32 - 1);

            if clicked_index != self.selected_index {
                // Start indicator animation
                self.indicator_start = self.indicator_anim;
                self.indicator_target = clicked_index as f32;
                self.indicator_progress = 0.0;

                self.selected_index = clicked_index;
                if let Some(ref cb) = self.on_change {
                    cb(clicked_index, &self.items[clicked_index as usize]);
                }
            }
        }

        // Animate indicator with EaseInOutBezier
        if self.indicator_progress < 1.0 {
            self.indicator_progress += dt / self.indicator_duration;
            if self.indicator_progress >= 1.0 {
                self.indicator_progress = 1.0;
            }
            let eased = ease_in_out_bezier(self.indicator_progress);
            self.indicator_anim = lerp_f32(self.indicator_start, self.indicator_target, eased);
        }

        self.base.request_visual_repaint();
    }

    fn draw(&self, renderer: &mut Renderer) {
        let p = &self.base.primitive;
        let frame = primitive_frame(p);
        let opacity = p.opacity;
        let palette = theme::current_theme_colors(&theme::dark_theme(), true);

        if self.items.is_empty() {
            return;
        }

        let rounding = frame.height * 0.5;
        let segment_w = self.segment_width();
        let indicator_inset = 3.0;
        let text_scale = self.font_size / 24.0;

        // Background
        let bg_color = palette.surface.apply_opacity(opacity);
        renderer.draw_rect_simple(frame.x, frame.y, frame.width, frame.height, &bg_color, rounding);

        // Animated indicator
        let ind_x = frame.x + self.indicator_anim * segment_w + indicator_inset;
        let ind_y = frame.y + indicator_inset;
        let ind_w = segment_w - indicator_inset * 2.0;
        let ind_h = frame.height - indicator_inset * 2.0;
        let ind_rounding = ind_h * 0.5;
        let ind_color = palette.primary.apply_opacity(opacity);
        renderer.draw_rect_simple(ind_x, ind_y, ind_w, ind_h, &ind_color, ind_rounding);

        // Text labels
        for (i, item) in self.items.iter().enumerate() {
            let seg_center_x = frame.x + (i as f32 + 0.5) * segment_w;
            let indicator_center = frame.x + (self.indicator_anim + 0.5) * segment_w;
            let distance = ((seg_center_x - indicator_center) / segment_w).abs();
            let proximity = (1.0 - distance).clamp(0.0, 1.0);
            let normal_text = palette.text.with_alpha(0.7).apply_opacity(opacity);
            let active_text = Color::WHITE.apply_opacity(opacity);
            let text_color = lerp_color(&normal_text, &active_text, proximity);

            // Center text in segment
            let text_w = renderer.measure_text_width(item, text_scale);
            let text_x = frame.x + i as f32 * segment_w + (segment_w - text_w) * 0.5;
            let text_y = frame.y + (frame.height - self.font_size) * 0.5;
            renderer.draw_text(item, text_x, text_y, &text_color, text_scale);
        }
    }

    fn reset_defaults(&mut self) {
        self.base.primitive.width = 300.0;
        self.base.primitive.height = 35.0;
    }

    fn wants_continuous_update(&self) -> bool {
        self.indicator_progress < 1.0
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
