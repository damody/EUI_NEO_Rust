//! PrimitiveMotion: loop and hover animation tracks for scalar and color properties.
//!
//! Translated from C++ PrimitiveMotion.h.

use crate::color::Color;
use crate::easing::Easing;
use crate::animation::{FloatAnimation, ColorAnimation};
use crate::ui::primitive::UIPrimitive;

// ---------------------------------------------------------------------------
// Motion spec types
// ---------------------------------------------------------------------------

/// Specification for a looping scalar (float) animation track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarMotionSpec {
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub easing: Easing,
    pub enabled: bool,
}

impl Default for ScalarMotionSpec {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            duration: 0.0,
            easing: Easing::EaseInOut,
            enabled: false,
        }
    }
}

/// Specification for a looping color animation track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorMotionSpec {
    pub from: Color,
    pub to: Color,
    pub duration: f32,
    pub easing: Easing,
    pub enabled: bool,
}

impl Default for ColorMotionSpec {
    fn default() -> Self {
        Self {
            from: Color::WHITE,
            to: Color::WHITE,
            duration: 0.0,
            easing: Easing::EaseInOut,
            enabled: false,
        }
    }
}

/// Specification for a hover-triggered scalar animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverScalarMotionSpec {
    pub target: f32,
    pub duration: f32,
    pub easing: Easing,
    pub enabled: bool,
}

impl Default for HoverScalarMotionSpec {
    fn default() -> Self {
        Self {
            target: 0.0,
            duration: 0.2,
            easing: Easing::EaseOut,
            enabled: false,
        }
    }
}

/// Specification for a hover-triggered color animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverColorMotionSpec {
    pub target: Color,
    pub duration: f32,
    pub easing: Easing,
    pub enabled: bool,
}

impl Default for HoverColorMotionSpec {
    fn default() -> Self {
        Self {
            target: Color::WHITE,
            duration: 0.2,
            easing: Easing::EaseOut,
            enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Free-standing helper functions (avoid borrow-checker issues with &self)
// ---------------------------------------------------------------------------

/// Reconcile a loop scalar track: if spec changed, start or clear the animation.
fn reconcile_scalar_track(
    new_spec: &ScalarMotionSpec,
    old_spec: &ScalarMotionSpec,
    anim: &mut FloatAnimation,
) {
    if *new_spec == *old_spec {
        return;
    }
    if new_spec.enabled {
        anim.play(new_spec.from, new_spec.to, new_spec.duration, new_spec.easing);
        // Queue the reverse leg for ping-pong looping
        anim.queue(new_spec.from, new_spec.duration, new_spec.easing);
    } else {
        anim.clear();
    }
}

/// Reconcile a loop color track.
fn reconcile_color_track(
    new_spec: &ColorMotionSpec,
    old_spec: &ColorMotionSpec,
    anim: &mut ColorAnimation,
) {
    if *new_spec == *old_spec {
        return;
    }
    if new_spec.enabled {
        anim.play(new_spec.from, new_spec.to, new_spec.duration, new_spec.easing);
        anim.queue(new_spec.from, new_spec.duration, new_spec.easing);
    } else {
        anim.clear();
    }
}

/// Reconcile a hover scalar track.
fn reconcile_hover_scalar_track(
    new_spec: &HoverScalarMotionSpec,
    old_spec: &HoverScalarMotionSpec,
    anim: &mut FloatAnimation,
    is_hovered: bool,
    active: &mut bool,
    rest_value: f32,
) {
    let spec_changed = *new_spec != *old_spec;
    let hover_changed = is_hovered != *active;

    if !new_spec.enabled {
        if spec_changed && old_spec.enabled {
            anim.clear();
            *active = false;
        }
        return;
    }

    if spec_changed || hover_changed {
        *active = is_hovered;
        if is_hovered {
            let from = if anim.is_active() {
                *anim.current()
            } else {
                rest_value
            };
            anim.play(from, new_spec.target, new_spec.duration, new_spec.easing);
        } else {
            let from = if anim.is_active() {
                *anim.current()
            } else {
                new_spec.target
            };
            anim.play(from, rest_value, new_spec.duration, new_spec.easing);
        }
    }
}

/// Reconcile a hover color track.
fn reconcile_hover_color_track(
    new_spec: &HoverColorMotionSpec,
    old_spec: &HoverColorMotionSpec,
    anim: &mut ColorAnimation,
    is_hovered: bool,
    active: &mut bool,
    rest_value: Color,
) {
    let spec_changed = *new_spec != *old_spec;
    let hover_changed = is_hovered != *active;

    if !new_spec.enabled {
        if spec_changed && old_spec.enabled {
            anim.clear();
            *active = false;
        }
        return;
    }

    if spec_changed || hover_changed {
        *active = is_hovered;
        if is_hovered {
            let from = if anim.is_active() {
                *anim.current()
            } else {
                rest_value
            };
            anim.play(from, new_spec.target, new_spec.duration, new_spec.easing);
        } else {
            let from = if anim.is_active() {
                *anim.current()
            } else {
                new_spec.target
            };
            anim.play(from, rest_value, new_spec.duration, new_spec.easing);
        }
    }
}

// ---------------------------------------------------------------------------
// PrimitiveMotionState
// ---------------------------------------------------------------------------

/// Tracks loop and hover animations for the six primitive properties:
/// scale, rotation, opacity, translateX, translateY, background.
pub struct PrimitiveMotionState {
    // --- Applied specs (last specs that were started) ---
    applied_scale_spec: ScalarMotionSpec,
    applied_rotation_spec: ScalarMotionSpec,
    applied_opacity_spec: ScalarMotionSpec,
    applied_translate_x_spec: ScalarMotionSpec,
    applied_translate_y_spec: ScalarMotionSpec,
    applied_background_spec: ColorMotionSpec,

    applied_hover_scale_spec: HoverScalarMotionSpec,
    applied_hover_rotation_spec: HoverScalarMotionSpec,
    applied_hover_opacity_spec: HoverScalarMotionSpec,
    applied_hover_translate_x_spec: HoverScalarMotionSpec,
    applied_hover_translate_y_spec: HoverScalarMotionSpec,
    applied_hover_background_spec: HoverColorMotionSpec,

    // --- Current values ---
    scale_value: f32,
    rotation_value: f32,
    opacity_value: f32,
    translate_x_value: f32,
    translate_y_value: f32,
    background_value: Color,

    // --- Hover active flags ---
    hover_scale_active: bool,
    hover_rotation_active: bool,
    hover_opacity_active: bool,
    hover_translate_x_active: bool,
    hover_translate_y_active: bool,
    hover_background_active: bool,

    // --- Animation objects ---
    scale_animation: FloatAnimation,
    rotation_animation: FloatAnimation,
    opacity_animation: FloatAnimation,
    translate_x_animation: FloatAnimation,
    translate_y_animation: FloatAnimation,
    background_animation: ColorAnimation,

    hover_scale_animation: FloatAnimation,
    hover_rotation_animation: FloatAnimation,
    hover_opacity_animation: FloatAnimation,
    hover_translate_x_animation: FloatAnimation,
    hover_translate_y_animation: FloatAnimation,
    hover_background_animation: ColorAnimation,
}

impl Default for PrimitiveMotionState {
    fn default() -> Self {
        Self {
            applied_scale_spec: ScalarMotionSpec::default(),
            applied_rotation_spec: ScalarMotionSpec::default(),
            applied_opacity_spec: ScalarMotionSpec::default(),
            applied_translate_x_spec: ScalarMotionSpec::default(),
            applied_translate_y_spec: ScalarMotionSpec::default(),
            applied_background_spec: ColorMotionSpec::default(),

            applied_hover_scale_spec: HoverScalarMotionSpec::default(),
            applied_hover_rotation_spec: HoverScalarMotionSpec::default(),
            applied_hover_opacity_spec: HoverScalarMotionSpec::default(),
            applied_hover_translate_x_spec: HoverScalarMotionSpec::default(),
            applied_hover_translate_y_spec: HoverScalarMotionSpec::default(),
            applied_hover_background_spec: HoverColorMotionSpec::default(),

            scale_value: 1.0,
            rotation_value: 0.0,
            opacity_value: 1.0,
            translate_x_value: 0.0,
            translate_y_value: 0.0,
            background_value: Color::WHITE,

            hover_scale_active: false,
            hover_rotation_active: false,
            hover_opacity_active: false,
            hover_translate_x_active: false,
            hover_translate_y_active: false,
            hover_background_active: false,

            scale_animation: FloatAnimation::new(),
            rotation_animation: FloatAnimation::new(),
            opacity_animation: FloatAnimation::new(),
            translate_x_animation: FloatAnimation::new(),
            translate_y_animation: FloatAnimation::new(),
            background_animation: ColorAnimation::new(),

            hover_scale_animation: FloatAnimation::new(),
            hover_rotation_animation: FloatAnimation::new(),
            hover_opacity_animation: FloatAnimation::new(),
            hover_translate_x_animation: FloatAnimation::new(),
            hover_translate_y_animation: FloatAnimation::new(),
            hover_background_animation: ColorAnimation::new(),
        }
    }
}

impl PrimitiveMotionState {
    pub fn new() -> Self {
        Self::default()
    }

    // -----------------------------------------------------------------------
    // Public: query whether any track needs continuous updates
    // -----------------------------------------------------------------------

    pub fn wants_continuous_update(&self) -> bool {
        self.scale_animation.is_active()
            || self.rotation_animation.is_active()
            || self.opacity_animation.is_active()
            || self.translate_x_animation.is_active()
            || self.translate_y_animation.is_active()
            || self.background_animation.is_active()
            || self.hover_scale_animation.is_active()
            || self.hover_rotation_animation.is_active()
            || self.hover_opacity_animation.is_active()
            || self.hover_translate_x_animation.is_active()
            || self.hover_translate_y_animation.is_active()
            || self.hover_background_animation.is_active()
    }

    // -----------------------------------------------------------------------
    // Public: per-frame update - reconcile specs, tick animations
    // -----------------------------------------------------------------------

    /// Call once per frame. `dt` is the delta time in seconds. `is_hovered`
    /// indicates whether the primitive is currently hovered by the pointer.
    ///
    /// The 6 loop specs and 6 hover specs are the *desired* specs set by the
    /// owning node each compose pass.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f32,
        is_hovered: bool,
        // loop specs
        scale_spec: &ScalarMotionSpec,
        rotation_spec: &ScalarMotionSpec,
        opacity_spec: &ScalarMotionSpec,
        translate_x_spec: &ScalarMotionSpec,
        translate_y_spec: &ScalarMotionSpec,
        background_spec: &ColorMotionSpec,
        // hover specs
        hover_scale_spec: &HoverScalarMotionSpec,
        hover_rotation_spec: &HoverScalarMotionSpec,
        hover_opacity_spec: &HoverScalarMotionSpec,
        hover_translate_x_spec: &HoverScalarMotionSpec,
        hover_translate_y_spec: &HoverScalarMotionSpec,
        hover_background_spec: &HoverColorMotionSpec,
    ) {
        // --- Reconcile loop tracks ---
        reconcile_scalar_track(scale_spec, &self.applied_scale_spec, &mut self.scale_animation);
        self.applied_scale_spec = *scale_spec;

        reconcile_scalar_track(rotation_spec, &self.applied_rotation_spec, &mut self.rotation_animation);
        self.applied_rotation_spec = *rotation_spec;

        reconcile_scalar_track(opacity_spec, &self.applied_opacity_spec, &mut self.opacity_animation);
        self.applied_opacity_spec = *opacity_spec;

        reconcile_scalar_track(translate_x_spec, &self.applied_translate_x_spec, &mut self.translate_x_animation);
        self.applied_translate_x_spec = *translate_x_spec;

        reconcile_scalar_track(translate_y_spec, &self.applied_translate_y_spec, &mut self.translate_y_animation);
        self.applied_translate_y_spec = *translate_y_spec;

        reconcile_color_track(background_spec, &self.applied_background_spec, &mut self.background_animation);
        self.applied_background_spec = *background_spec;

        // --- Reconcile hover tracks ---
        reconcile_hover_scalar_track(
            hover_scale_spec,
            &self.applied_hover_scale_spec,
            &mut self.hover_scale_animation,
            is_hovered,
            &mut self.hover_scale_active,
            1.0, // rest value for scale
        );
        self.applied_hover_scale_spec = *hover_scale_spec;

        reconcile_hover_scalar_track(
            hover_rotation_spec,
            &self.applied_hover_rotation_spec,
            &mut self.hover_rotation_animation,
            is_hovered,
            &mut self.hover_rotation_active,
            0.0,
        );
        self.applied_hover_rotation_spec = *hover_rotation_spec;

        reconcile_hover_scalar_track(
            hover_opacity_spec,
            &self.applied_hover_opacity_spec,
            &mut self.hover_opacity_animation,
            is_hovered,
            &mut self.hover_opacity_active,
            1.0,
        );
        self.applied_hover_opacity_spec = *hover_opacity_spec;

        reconcile_hover_scalar_track(
            hover_translate_x_spec,
            &self.applied_hover_translate_x_spec,
            &mut self.hover_translate_x_animation,
            is_hovered,
            &mut self.hover_translate_x_active,
            0.0,
        );
        self.applied_hover_translate_x_spec = *hover_translate_x_spec;

        reconcile_hover_scalar_track(
            hover_translate_y_spec,
            &self.applied_hover_translate_y_spec,
            &mut self.hover_translate_y_animation,
            is_hovered,
            &mut self.hover_translate_y_active,
            0.0,
        );
        self.applied_hover_translate_y_spec = *hover_translate_y_spec;

        reconcile_hover_color_track(
            hover_background_spec,
            &self.applied_hover_background_spec,
            &mut self.hover_background_animation,
            is_hovered,
            &mut self.hover_background_active,
            Color::WHITE,
        );
        self.applied_hover_background_spec = *hover_background_spec;

        // --- Tick all animations ---
        self.scale_animation.update(dt);
        self.rotation_animation.update(dt);
        self.opacity_animation.update(dt);
        self.translate_x_animation.update(dt);
        self.translate_y_animation.update(dt);
        self.background_animation.update(dt);

        self.hover_scale_animation.update(dt);
        self.hover_rotation_animation.update(dt);
        self.hover_opacity_animation.update(dt);
        self.hover_translate_x_animation.update(dt);
        self.hover_translate_y_animation.update(dt);
        self.hover_background_animation.update(dt);

        // --- Read back current values from active animations ---
        if self.scale_animation.is_active() {
            self.scale_value = *self.scale_animation.current();
        }
        if self.rotation_animation.is_active() {
            self.rotation_value = *self.rotation_animation.current();
        }
        if self.opacity_animation.is_active() {
            self.opacity_value = *self.opacity_animation.current();
        }
        if self.translate_x_animation.is_active() {
            self.translate_x_value = *self.translate_x_animation.current();
        }
        if self.translate_y_animation.is_active() {
            self.translate_y_value = *self.translate_y_animation.current();
        }
        if self.background_animation.is_active() {
            self.background_value = *self.background_animation.current();
        }
    }

    // -----------------------------------------------------------------------
    // Public: apply motion values onto a UIPrimitive (mutates in place)
    // -----------------------------------------------------------------------

    /// Apply the current motion values to the given primitive.
    /// Loop track values are *multiplied* (scale, opacity) or *added*
    /// (rotation, translateX/Y) to the primitive fields.
    /// Hover track values are layered on top.
    pub fn apply(&self, prim: &mut UIPrimitive) {
        // Loop tracks
        prim.scale_x *= self.scale_value;
        prim.scale_y *= self.scale_value;
        prim.rotation += self.rotation_value;
        prim.opacity *= self.opacity_value;
        prim.translate_x += self.translate_x_value;
        prim.translate_y += self.translate_y_value;

        // Background: if the loop spec is enabled, override the background.
        if self.applied_background_spec.enabled {
            prim.background = self.background_value;
        }

        // Hover tracks (additive / multiplicative on top)
        if self.applied_hover_scale_spec.enabled {
            let hover_scale = *self.hover_scale_animation.current();
            prim.scale_x *= hover_scale;
            prim.scale_y *= hover_scale;
        }
        if self.applied_hover_rotation_spec.enabled {
            prim.rotation += *self.hover_rotation_animation.current();
        }
        if self.applied_hover_opacity_spec.enabled {
            prim.opacity *= *self.hover_opacity_animation.current();
        }
        if self.applied_hover_translate_x_spec.enabled {
            prim.translate_x += *self.hover_translate_x_animation.current();
        }
        if self.applied_hover_translate_y_spec.enabled {
            prim.translate_y += *self.hover_translate_y_animation.current();
        }
        if self.applied_hover_background_spec.enabled {
            prim.background = *self.hover_background_animation.current();
        }
    }

    // -----------------------------------------------------------------------
    // Accessors for current motion values
    // -----------------------------------------------------------------------

    pub fn scale_value(&self) -> f32 {
        self.scale_value
    }

    pub fn rotation_value(&self) -> f32 {
        self.rotation_value
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity_value
    }

    pub fn translate_x_value(&self) -> f32 {
        self.translate_x_value
    }

    pub fn translate_y_value(&self) -> f32 {
        self.translate_y_value
    }

    pub fn background_value(&self) -> Color {
        self.background_value
    }
}
