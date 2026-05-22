//! PolygonNode -- filled polygon UI element with motion support.
//!
//! Translated from C++ Polygon.h.

use std::any::Any;

use crate::rect::Point2;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::types::RectGradient;
use crate::ui::node::{UINode, NodeBase};
use crate::ui::primitive::{UIPrimitive, primitive_frame, apply_opacity};

use super::primitive_motion::{
    PrimitiveMotionState, ScalarMotionSpec, ColorMotionSpec,
    HoverScalarMotionSpec, HoverColorMotionSpec,
};

// ---------------------------------------------------------------------------
// PolygonNode
// ---------------------------------------------------------------------------

/// A filled polygon defined by a set of normalized points (0..1 range),
/// mapped to the primitive's screen rect. Supports the same 12 motion
/// specs as PanelNode.
pub struct PolygonNode {
    pub base: NodeBase,

    /// Polygon vertices in normalized coordinates (0..1 relative to the
    /// primitive's width/height).
    pub points: Vec<Point2>,

    // Motion state
    pub motion: PrimitiveMotionState,

    // Loop motion specs
    pub scale_spec: ScalarMotionSpec,
    pub rotation_spec: ScalarMotionSpec,
    pub opacity_spec: ScalarMotionSpec,
    pub translate_x_spec: ScalarMotionSpec,
    pub translate_y_spec: ScalarMotionSpec,
    pub background_spec: ColorMotionSpec,

    // Hover motion specs
    pub hover_scale_spec: HoverScalarMotionSpec,
    pub hover_rotation_spec: HoverScalarMotionSpec,
    pub hover_opacity_spec: HoverScalarMotionSpec,
    pub hover_translate_x_spec: HoverScalarMotionSpec,
    pub hover_translate_y_spec: HoverScalarMotionSpec,
    pub hover_background_spec: HoverColorMotionSpec,
}

impl PolygonNode {
    pub fn new(key: String) -> Self {
        Self {
            base: NodeBase::new(key),
            points: Vec::new(),
            motion: PrimitiveMotionState::new(),
            scale_spec: ScalarMotionSpec::default(),
            rotation_spec: ScalarMotionSpec::default(),
            opacity_spec: ScalarMotionSpec::default(),
            translate_x_spec: ScalarMotionSpec::default(),
            translate_y_spec: ScalarMotionSpec::default(),
            background_spec: ColorMotionSpec::default(),
            hover_scale_spec: HoverScalarMotionSpec::default(),
            hover_rotation_spec: HoverScalarMotionSpec::default(),
            hover_opacity_spec: HoverScalarMotionSpec::default(),
            hover_translate_x_spec: HoverScalarMotionSpec::default(),
            hover_translate_y_spec: HoverScalarMotionSpec::default(),
            hover_background_spec: HoverColorMotionSpec::default(),
        }
    }

    /// Return a copy of the primitive with motion values applied.
    pub fn resolved_primitive(&self) -> UIPrimitive {
        let mut p = self.base.primitive.clone();
        self.motion.apply(&mut p);
        p
    }

    /// Transform normalized polygon points to screen coordinates using the
    /// resolved primitive's position, size, rotation, and scale.
    ///
    /// Each normalized point (nx, ny) in [0,1] is mapped to:
    ///   px = frame.x + nx * frame.width
    ///   py = frame.y + ny * frame.height
    /// then rotation and scale are applied around the polygon center.
    pub fn make_screen_points(&self) -> Vec<Point2> {
        let resolved = self.resolved_primitive();
        let frame = primitive_frame(&resolved);

        // Center of the polygon in screen space
        let cx = frame.x + frame.width * 0.5;
        let cy = frame.y + frame.height * 0.5;

        let rotation_rad = resolved.rotation.to_radians();
        let cos_r = rotation_rad.cos();
        let sin_r = rotation_rad.sin();
        let sx = resolved.scale_x;
        let sy = resolved.scale_y;

        self.points
            .iter()
            .map(|pt| {
                // Map normalized to screen
                let px = frame.x + pt.x * frame.width;
                let py = frame.y + pt.y * frame.height;

                // Translate to center-relative
                let dx = px - cx;
                let dy = py - cy;

                // Apply scale
                let sdx = dx * sx;
                let sdy = dy * sy;

                // Apply rotation
                let rx = sdx * cos_r - sdy * sin_r;
                let ry = sdx * sin_r + sdy * cos_r;

                // Translate back and add motion translate
                Point2 {
                    x: rx + cx + resolved.translate_x,
                    y: ry + cy + resolved.translate_y,
                }
            })
            .collect()
    }

    /// Ray-casting point-in-polygon test.
    /// `px`, `py` are in screen coordinates.
    /// `screen_pts` should be the result of `make_screen_points()`.
    pub fn contains_point_in_polygon(px: f32, py: f32, screen_pts: &[Point2]) -> bool {
        let n = screen_pts.len();
        if n < 3 {
            return false;
        }

        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let pi = &screen_pts[i];
            let pj = &screen_pts[j];

            // Ray cast: count crossings of a horizontal ray from (px, py) to the right
            if ((pi.y > py) != (pj.y > py))
                && (px < (pj.x - pi.x) * (py - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Convenience: test whether a screen-space point lies inside this polygon.
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        let screen_pts = self.make_screen_points();
        Self::contains_point_in_polygon(px, py, &screen_pts)
    }
}

impl UINode for PolygonNode {
    fn type_name(&self) -> &'static str {
        "Polygon"
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

        // Hover test using the polygon shape
        let is_hovered = self.contains_point(state.mouse_x, state.mouse_y);

        self.motion.update(
            dt,
            is_hovered,
            &self.scale_spec,
            &self.rotation_spec,
            &self.opacity_spec,
            &self.translate_x_spec,
            &self.translate_y_spec,
            &self.background_spec,
            &self.hover_scale_spec,
            &self.hover_rotation_spec,
            &self.hover_opacity_spec,
            &self.hover_translate_x_spec,
            &self.hover_translate_y_spec,
            &self.hover_background_spec,
        );
    }

    fn draw(&self, renderer: &mut Renderer) {
        let resolved = self.resolved_primitive();
        if !resolved.visible || resolved.opacity <= 0.001 {
            return;
        }
        if self.points.len() < 3 {
            return;
        }

        let screen_pts = self.make_screen_points();
        let fill_color = apply_opacity(resolved.background, resolved.opacity);
        let gradient = if resolved.gradient.enabled {
            let op = resolved.opacity;
            RectGradient {
                enabled: true,
                top_left: apply_opacity(resolved.gradient.top_left, op),
                top_right: apply_opacity(resolved.gradient.top_right, op),
                bottom_left: apply_opacity(resolved.gradient.bottom_left, op),
                bottom_right: apply_opacity(resolved.gradient.bottom_right, op),
            }
        } else {
            resolved.gradient
        };

        let stroke_color = apply_opacity(resolved.border_color, resolved.opacity);
        let stroke_width = resolved.border_width;

        renderer.draw_polygon(&screen_pts, &fill_color, &gradient, stroke_width, &stroke_color);
    }

    fn reset_defaults(&mut self) {
        // Points are preserved across composes; only motion specs are reset.
        self.scale_spec = ScalarMotionSpec::default();
        self.rotation_spec = ScalarMotionSpec::default();
        self.opacity_spec = ScalarMotionSpec::default();
        self.translate_x_spec = ScalarMotionSpec::default();
        self.translate_y_spec = ScalarMotionSpec::default();
        self.background_spec = ColorMotionSpec::default();
        self.hover_scale_spec = HoverScalarMotionSpec::default();
        self.hover_rotation_spec = HoverScalarMotionSpec::default();
        self.hover_opacity_spec = HoverScalarMotionSpec::default();
        self.hover_translate_x_spec = HoverScalarMotionSpec::default();
        self.hover_translate_y_spec = HoverScalarMotionSpec::default();
        self.hover_background_spec = HoverColorMotionSpec::default();
    }

    fn wants_continuous_update(&self) -> bool {
        self.motion.wants_continuous_update()
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
