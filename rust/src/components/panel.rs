//! PanelNode, GlassPanelNode, PopupPanelNode -- rect-based UI panels.
//!
//! Translated from C++ Panel.h.

use std::any::Any;

use crate::color::Color;
use crate::state::UIState;
use crate::types::{RectStyle, RenderLayer, UIShadow};
use crate::ui::node::{UINode, NodeBase};
use crate::ui::primitive::{UIPrimitive, primitive_frame, make_style, apply_opacity};
use crate::renderer::renderer::Renderer;

use super::primitive_motion::{
    PrimitiveMotionState, ScalarMotionSpec, ColorMotionSpec,
    HoverScalarMotionSpec, HoverColorMotionSpec,
};

// ---------------------------------------------------------------------------
// draw_panel helper
// ---------------------------------------------------------------------------

/// Draws a panel rectangle with an optional border outline.
/// This is the shared rendering path for all panel variants.
fn draw_panel(prim: &UIPrimitive, renderer: &mut Renderer) {
    let frame = primitive_frame(prim);
    let style = make_style(prim);

    // Border (drawn behind main rect as a slightly larger rect)
    if prim.border_width > 0.0 && prim.border_color.a > 0.001 {
        let bw = prim.border_width;
        let border_style = RectStyle {
            color: apply_opacity(prim.border_color, prim.opacity),
            rounding: prim.rounding + bw,
            ..RectStyle::default()
        };
        renderer.draw_rect(frame.x - bw, frame.y - bw, frame.width + bw * 2.0, frame.height + bw * 2.0, &border_style);
    }

    renderer.draw_rect(frame.x, frame.y, frame.width, frame.height, &style);
}

// ---------------------------------------------------------------------------
// PanelNode
// ---------------------------------------------------------------------------

/// A rectangular panel with background, border, shadow, and motion animations.
pub struct PanelNode {
    pub base: NodeBase,

    // Motion state
    pub motion: PrimitiveMotionState,

    // Loop motion specs (set each compose)
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

impl PanelNode {
    pub fn new(key: String) -> Self {
        Self {
            base: NodeBase::new(key),
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

    /// Check if the given screen-space point is inside this panel.
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let frame = primitive_frame(&self.resolved_primitive());
        frame.contains(x, y)
    }
}

impl UINode for PanelNode {
    fn type_name(&self) -> &'static str {
        "Panel"
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

        // Simple hover test using the raw primitive frame
        let frame = primitive_frame(&self.base.primitive);
        let is_hovered = frame.contains(state.mouse_x, state.mouse_y);

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
        draw_panel(&resolved, renderer);
    }

    fn reset_defaults(&mut self) {
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

// ---------------------------------------------------------------------------
// GlassPanelNode
// ---------------------------------------------------------------------------

/// A translucent "glass" panel with default semi-transparent background,
/// large rounding, and a subtle shadow.
pub struct GlassPanelNode {
    pub base: NodeBase,
    pub motion: PrimitiveMotionState,

    pub scale_spec: ScalarMotionSpec,
    pub rotation_spec: ScalarMotionSpec,
    pub opacity_spec: ScalarMotionSpec,
    pub translate_x_spec: ScalarMotionSpec,
    pub translate_y_spec: ScalarMotionSpec,
    pub background_spec: ColorMotionSpec,

    pub hover_scale_spec: HoverScalarMotionSpec,
    pub hover_rotation_spec: HoverScalarMotionSpec,
    pub hover_opacity_spec: HoverScalarMotionSpec,
    pub hover_translate_x_spec: HoverScalarMotionSpec,
    pub hover_translate_y_spec: HoverScalarMotionSpec,
    pub hover_background_spec: HoverColorMotionSpec,
}

impl GlassPanelNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        // Glass defaults
        base.primitive.background = Color::new(1.0, 1.0, 1.0, 0.35);
        base.primitive.rounding = 18.0;
        base.primitive.shadow = UIShadow {
            blur: 12.0,
            offset_x: 0.0,
            offset_y: 4.0,
            color: Color::new(0.0, 0.0, 0.0, 0.10),
        };
        base.primitive.blur = 8.0;

        Self {
            base,
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

    pub fn resolved_primitive(&self) -> UIPrimitive {
        let mut p = self.base.primitive.clone();
        self.motion.apply(&mut p);
        p
    }
}

impl UINode for GlassPanelNode {
    fn type_name(&self) -> &'static str {
        "GlassPanel"
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
        let is_hovered = frame.contains(state.mouse_x, state.mouse_y);

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
        draw_panel(&resolved, renderer);
    }

    fn reset_defaults(&mut self) {
        // Restore glass defaults
        self.base.primitive.background = Color::new(1.0, 1.0, 1.0, 0.35);
        self.base.primitive.rounding = 18.0;
        self.base.primitive.shadow = UIShadow {
            blur: 12.0,
            offset_x: 0.0,
            offset_y: 4.0,
            color: Color::new(0.0, 0.0, 0.0, 0.10),
        };
        self.base.primitive.blur = 8.0;

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

// ---------------------------------------------------------------------------
// PopupPanelNode
// ---------------------------------------------------------------------------

/// A popup panel with elevated z-index, shadow, and moderate rounding.
pub struct PopupPanelNode {
    pub base: NodeBase,
    pub motion: PrimitiveMotionState,

    pub scale_spec: ScalarMotionSpec,
    pub rotation_spec: ScalarMotionSpec,
    pub opacity_spec: ScalarMotionSpec,
    pub translate_x_spec: ScalarMotionSpec,
    pub translate_y_spec: ScalarMotionSpec,
    pub background_spec: ColorMotionSpec,

    pub hover_scale_spec: HoverScalarMotionSpec,
    pub hover_rotation_spec: HoverScalarMotionSpec,
    pub hover_opacity_spec: HoverScalarMotionSpec,
    pub hover_translate_x_spec: HoverScalarMotionSpec,
    pub hover_translate_y_spec: HoverScalarMotionSpec,
    pub hover_background_spec: HoverColorMotionSpec,
}

impl PopupPanelNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        // Popup defaults
        base.primitive.background = Color::WHITE;
        base.primitive.rounding = 14.0;
        base.primitive.render_layer = RenderLayer::Popup;
        base.primitive.z_index = 100;
        base.primitive.shadow = UIShadow {
            blur: 20.0,
            offset_x: 0.0,
            offset_y: 8.0,
            color: Color::new(0.0, 0.0, 0.0, 0.18),
        };

        Self {
            base,
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

    pub fn resolved_primitive(&self) -> UIPrimitive {
        let mut p = self.base.primitive.clone();
        self.motion.apply(&mut p);
        p
    }
}

impl UINode for PopupPanelNode {
    fn type_name(&self) -> &'static str {
        "PopupPanel"
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
        let is_hovered = frame.contains(state.mouse_x, state.mouse_y);

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
        draw_panel(&resolved, renderer);
    }

    fn reset_defaults(&mut self) {
        // Restore popup defaults
        self.base.primitive.background = Color::WHITE;
        self.base.primitive.rounding = 14.0;
        self.base.primitive.render_layer = RenderLayer::Popup;
        self.base.primitive.z_index = 100;
        self.base.primitive.shadow = UIShadow {
            blur: 20.0,
            offset_x: 0.0,
            offset_y: 8.0,
            color: Color::new(0.0, 0.0, 0.0, 0.18),
        };

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
