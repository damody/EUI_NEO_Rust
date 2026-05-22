use std::any::Any;
use crate::state::UIState;
use crate::rect::RectFrame;
use crate::ui::primitive::{UIPrimitive, primitive_frame};
use crate::renderer::renderer::Renderer;

/// Trait representing a retained UI node (replaces C++ UINode virtual class).
pub trait UINode: Any {
    /// Static type name for this node kind (e.g. "Panel", "Label").
    fn type_name(&self) -> &'static str;

    /// Unique key identifying this node instance within a UIContext.
    fn key(&self) -> &str;

    /// Immutable access to the underlying primitive.
    fn primitive(&self) -> &UIPrimitive;

    /// Mutable access to the underlying primitive.
    fn primitive_mut(&mut self) -> &mut UIPrimitive;

    /// Per-frame update (animations, hover state, etc.).
    fn update(&mut self, state: &UIState);

    /// Emit draw commands for this node.
    fn draw(&self, renderer: &mut Renderer);

    /// Reset node-specific properties to defaults (called at the start of each compose).
    fn reset_defaults(&mut self);

    /// Layout bounds (local space, origin at 0,0).
    fn layout_bounds(&self) -> RectFrame {
        let p = self.primitive();
        RectFrame {
            x: 0.0,
            y: 0.0,
            width: p.width,
            height: p.height,
        }
    }

    /// Paint bounds (screen space, includes context offset).
    fn paint_bounds(&self) -> RectFrame {
        let p = self.primitive();
        primitive_frame(p)
    }

    /// Whether this node requires continuous updates (e.g. running animations).
    fn wants_continuous_update(&self) -> bool {
        false
    }

    /// Whether this node can use a cached surface for drawing.
    fn uses_cached_surface(&self) -> bool {
        true
    }

    // -- Compose tracking --

    /// Begin a compose pass with the given stamp.
    fn begin_compose(&mut self, stamp: u64);

    /// Finish a compose pass, snapshotting the current state.
    fn finish_compose(&mut self);

    /// Was this node composed in the given stamp?
    fn composed_in(&self, stamp: u64) -> bool;

    /// Is the visual cache dirty (needs redraw)?
    fn cache_dirty(&self) -> bool;

    /// Clear the cache dirty flag after a redraw.
    fn clear_cache_dirty(&mut self);

    /// Downcast to Any for type-safe access.
    fn as_any(&self) -> &dyn Any;

    /// Downcast to Any (mutable) for type-safe access.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// ---------------------------------------------------------------------------
// NodeBase: shared compose-tracking data for all node implementations
// ---------------------------------------------------------------------------

/// Base data for compose tracking, shared by all node implementations.
#[derive(Debug, Clone)]
pub struct NodeBase {
    pub key: String,
    pub primitive: UIPrimitive,
    pub compose_stamp: u64,
    pub compose_build_hash: u64,
    pub composed_hash: u64,
    pub composed_primitive: UIPrimitive,
    pub has_compose_snapshot: bool,
    pub cache_dirty: bool,
    pub recompose_requested: bool,
}

const COMPOSE_HASH_SEED: u64 = 1469598103934665603;
const COMPOSE_EPSILON: f32 = 0.0001;

impl NodeBase {
    pub fn new(key: String) -> Self {
        Self {
            key,
            primitive: UIPrimitive::default(),
            compose_stamp: 0,
            compose_build_hash: COMPOSE_HASH_SEED,
            composed_hash: 0,
            composed_primitive: UIPrimitive::default(),
            has_compose_snapshot: false,
            cache_dirty: true,
            recompose_requested: false,
        }
    }

    /// Begin a compose pass: record stamp, reset hash, clear recompose flag.
    pub fn begin_compose(&mut self, stamp: u64) {
        self.compose_stamp = stamp;
        self.compose_build_hash = COMPOSE_HASH_SEED;
        self.recompose_requested = false;
    }

    /// Finish a compose pass: compare hash and primitive, mark dirty if changed.
    pub fn finish_compose(&mut self) {
        let primitive_changed = !primitive_eq(&self.primitive, &self.composed_primitive);
        let hash_changed = self.compose_build_hash != self.composed_hash;

        if !self.has_compose_snapshot || primitive_changed || hash_changed {
            self.cache_dirty = true;
        }

        self.composed_primitive = self.primitive.clone();
        self.composed_hash = self.compose_build_hash;
        self.has_compose_snapshot = true;
    }

    /// Was this node composed in the given stamp?
    pub fn composed_in(&self, stamp: u64) -> bool {
        self.compose_stamp == stamp
    }

    /// Hash raw bytes into the compose build hash (FNV-1a variant).
    pub fn track_compose_bytes(&mut self, data: &[u8]) {
        for &byte in data {
            self.compose_build_hash ^= byte as u64;
            self.compose_build_hash = self.compose_build_hash.wrapping_mul(1099511628211);
        }
    }

    /// Hash a tagged value into the compose build hash.
    pub fn track_compose_value<T: AsComposeBytes + ?Sized>(&mut self, tag: &str, value: &T) {
        self.track_compose_bytes(tag.as_bytes());
        self.track_compose_bytes(value.as_compose_bytes());
    }

    /// Hash a marker tag (no value) into the compose build hash.
    pub fn track_compose_marker(&mut self, tag: &str) {
        self.track_compose_bytes(tag.as_bytes());
    }

    /// Force the visual cache to be considered dirty.
    pub fn force_compose_dirty(&mut self) {
        self.cache_dirty = true;
    }

    /// Request a visual repaint (marks cache dirty).
    pub fn request_visual_repaint(&mut self) {
        self.cache_dirty = true;
    }

    /// Request a full compose rebuild on the next frame.
    pub fn request_compose_rebuild(&mut self) {
        self.recompose_requested = true;
        self.cache_dirty = true;
    }

    /// Consume and return the recompose request flag.
    pub fn consume_recompose_request(&mut self) -> bool {
        if !self.recompose_requested {
            return false;
        }
        self.recompose_requested = false;
        true
    }
}

// ---------------------------------------------------------------------------
// AsComposeBytes: trait for types that can produce bytes for compose hashing
// ---------------------------------------------------------------------------

/// Trait for types that can be hashed for compose tracking.
pub trait AsComposeBytes {
    fn as_compose_bytes(&self) -> &[u8];
}

impl AsComposeBytes for f32 {
    fn as_compose_bytes(&self) -> &[u8] {
        let bytes: &[u8; 4] = unsafe { &*(self as *const f32 as *const [u8; 4]) };
        bytes
    }
}

impl AsComposeBytes for f64 {
    fn as_compose_bytes(&self) -> &[u8] {
        let bytes: &[u8; 8] = unsafe { &*(self as *const f64 as *const [u8; 8]) };
        bytes
    }
}

impl AsComposeBytes for i32 {
    fn as_compose_bytes(&self) -> &[u8] {
        let bytes: &[u8; 4] = unsafe { &*(self as *const i32 as *const [u8; 4]) };
        bytes
    }
}

impl AsComposeBytes for u32 {
    fn as_compose_bytes(&self) -> &[u8] {
        let bytes: &[u8; 4] = unsafe { &*(self as *const u32 as *const [u8; 4]) };
        bytes
    }
}

impl AsComposeBytes for u64 {
    fn as_compose_bytes(&self) -> &[u8] {
        let bytes: &[u8; 8] = unsafe { &*(self as *const u64 as *const [u8; 8]) };
        bytes
    }
}

impl AsComposeBytes for bool {
    fn as_compose_bytes(&self) -> &[u8] {
        if *self { &[1] } else { &[0] }
    }
}

impl AsComposeBytes for str {
    fn as_compose_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsComposeBytes for String {
    fn as_compose_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsComposeBytes for [u8] {
    fn as_compose_bytes(&self) -> &[u8] {
        self
    }
}

// ---------------------------------------------------------------------------
// Primitive equality with epsilon (matches C++ PrimitiveEq)
// ---------------------------------------------------------------------------

use crate::color::Color;
use crate::types::{RectGradient, UIShadow};
use crate::ui::primitive::UIClipRect;

fn float_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= COMPOSE_EPSILON
}

fn color_eq(a: &Color, b: &Color) -> bool {
    float_eq(a.r, b.r) && float_eq(a.g, b.g) && float_eq(a.b, b.b) && float_eq(a.a, b.a)
}

fn gradient_eq(a: &RectGradient, b: &RectGradient) -> bool {
    a.enabled == b.enabled
        && color_eq(&a.top_left, &b.top_left)
        && color_eq(&a.top_right, &b.top_right)
        && color_eq(&a.bottom_left, &b.bottom_left)
        && color_eq(&a.bottom_right, &b.bottom_right)
}

fn shadow_eq(a: &UIShadow, b: &UIShadow) -> bool {
    float_eq(a.blur, b.blur)
        && float_eq(a.offset_x, b.offset_x)
        && float_eq(a.offset_y, b.offset_y)
        && color_eq(&a.color, &b.color)
}

fn clip_eq(a: &UIClipRect, b: &UIClipRect) -> bool {
    float_eq(a.x, b.x) && float_eq(a.y, b.y) && float_eq(a.width, b.width) && float_eq(a.height, b.height)
}

fn primitive_eq(a: &UIPrimitive, b: &UIPrimitive) -> bool {
    float_eq(a.x, b.x)
        && float_eq(a.y, b.y)
        && float_eq(a.context_offset_x, b.context_offset_x)
        && float_eq(a.context_offset_y, b.context_offset_y)
        && float_eq(a.width, b.width)
        && float_eq(a.height, b.height)
        && float_eq(a.min_width, b.min_width)
        && float_eq(a.min_height, b.min_height)
        && float_eq(a.max_width, b.max_width)
        && float_eq(a.max_height, b.max_height)
        && float_eq(a.scale_x, b.scale_x)
        && float_eq(a.scale_y, b.scale_y)
        && float_eq(a.rotation, b.rotation)
        && float_eq(a.translate_x, b.translate_x)
        && float_eq(a.translate_y, b.translate_y)
        && a.anchor == b.anchor
        && float_eq(a.rounding, b.rounding)
        && color_eq(&a.background, &b.background)
        && gradient_eq(&a.gradient, &b.gradient)
        && float_eq(a.border_width, b.border_width)
        && color_eq(&a.border_color, &b.border_color)
        && float_eq(a.blur, b.blur)
        && shadow_eq(&a.shadow, &b.shadow)
        && float_eq(a.opacity, b.opacity)
        && a.visible == b.visible
        && a.enabled == b.enabled
        && a.render_layer == b.render_layer
        && a.z_index == b.z_index
        && a.clip_to_parent == b.clip_to_parent
        && a.has_clip_rect == b.has_clip_rect
        && (!a.has_clip_rect || clip_eq(&a.clip_rect, &b.clip_rect))
}
