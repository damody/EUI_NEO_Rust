use std::any::Any;

use crate::color::{Color, lerp_color};
use crate::animation::FloatAnimation;
use crate::easing::Easing;
use crate::rect::RectFrame;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::theme;
use crate::types::UIShadow;
use crate::ui::node::{NodeBase, UINode};
use crate::ui::primitive::{UIPrimitive, primitive_frame};

/// Inline f32 lerp helper.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Specification for a sidebar item.
pub struct ItemSpec {
    pub icon: String,
    pub label: String,
    pub on_click: Option<Box<dyn Fn()>>,
}

/// SidebarNode -- vertical navigation sidebar with brand, items, and theme toggle.
pub struct SidebarNode {
    pub base: NodeBase,

    // --- configuration ---
    pub brand_primary: String,
    pub brand_secondary: String,
    pub selected_index: i32,
    pub collapsed_width: f32,
    pub expanded_width: f32,
    pub items: Vec<ItemSpec>,
    pub on_theme_toggle: Option<Box<dyn Fn()>>,

    // --- runtime ---
    pub item_hover: Vec<f32>,
    pub selection_anim: f32,
    pub selection_ready: bool,
    pub theme_hover: f32,
    pub theme_pressed: f32,
    pub theme_rotation: f32,
    pub theme_rotation_animation: FloatAnimation,
    pub theme_blend: f32,
    pub theme_blend_animation: FloatAnimation,
}

impl SidebarNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        base.primitive.width = 200.0;
        base.primitive.height = 300.0;
        base.primitive.rounding = 20.0;
        base.primitive.border_width = 1.0;
        base.primitive.border_color = Color::new(0.3, 0.3, 0.3, 0.5);
        base.primitive.shadow = UIShadow {
            blur: 12.0,
            offset_x: 0.0,
            offset_y: 4.0,
            color: Color::new(0.0, 0.0, 0.0, 0.15),
        };
        Self {
            base,
            brand_primary: String::new(),
            brand_secondary: String::new(),
            selected_index: 0,
            collapsed_width: 60.0,
            expanded_width: 200.0,
            items: Vec::new(),
            on_theme_toggle: None,
            item_hover: Vec::new(),
            selection_anim: 0.0,
            selection_ready: false,
            theme_hover: 0.0,
            theme_pressed: 0.0,
            theme_rotation: 0.0,
            theme_rotation_animation: FloatAnimation::new(),
            theme_blend: 0.0,
            theme_blend_animation: FloatAnimation::new(),
        }
    }

    /// Compute the rect for an item row at index i.
    fn item_rect(&self, i: usize) -> RectFrame {
        let frame = primitive_frame(&self.base.primitive);
        let item_height = 44.0;
        let top_offset = 80.0; // space for brand area
        let item_x = frame.x + 8.0;
        let item_y = frame.y + top_offset + i as f32 * item_height;
        let item_w = frame.width - 16.0;
        RectFrame::new(item_x, item_y, item_w, item_height)
    }

    /// Compute the rect for the theme toggle button.
    fn theme_toggle_rect(&self) -> RectFrame {
        let frame = primitive_frame(&self.base.primitive);
        let btn_size = 36.0;
        let btn_x = frame.x + (frame.width - btn_size) * 0.5;
        let btn_y = frame.y + frame.height - btn_size - 12.0;
        RectFrame::new(btn_x, btn_y, btn_size, btn_size)
    }
}

impl UINode for SidebarNode {
    fn type_name(&self) -> &'static str {
        "Sidebar"
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

        // Ensure item_hover matches items count
        self.item_hover.resize(self.items.len(), 0.0);

        // Initialize selection animation
        if !self.selection_ready && !self.items.is_empty() {
            self.selection_anim = self.selected_index as f32;
            self.selection_ready = true;
        }

        // Selection animation (smooth follow)
        let sel_target = self.selected_index as f32;
        self.selection_anim = lerp_f32(self.selection_anim, sel_target, dt * 12.0);

        // Item hover and click
        for i in 0..self.items.len() {
            let rect = self.item_rect(i);
            let hovered = rect.contains(state.mouse_x, state.mouse_y);
            let target = if hovered { 1.0 } else { 0.0 };
            self.item_hover[i] = lerp_f32(self.item_hover[i], target, dt * 10.0);

            if state.mouse_clicked && hovered {
                if self.selected_index != i as i32 {
                    self.selected_index = i as i32;
                }
                if let Some(ref cb) = self.items[i].on_click {
                    cb();
                }
            }
        }

        // Theme toggle button
        let toggle_rect = self.theme_toggle_rect();
        let toggle_hovered = toggle_rect.contains(state.mouse_x, state.mouse_y);
        let toggle_hover_target = if toggle_hovered { 1.0 } else { 0.0 };
        self.theme_hover = lerp_f32(self.theme_hover, toggle_hover_target, dt * 10.0);

        let toggle_pressed_target = if toggle_hovered && state.mouse_down { 1.0 } else { 0.0 };
        self.theme_pressed = lerp_f32(self.theme_pressed, toggle_pressed_target, dt * 14.0);

        if state.mouse_clicked && toggle_hovered {
            // Queue rotation animation (360 degrees)
            let current_rot = *self.theme_rotation_animation.current();
            self.theme_rotation_animation.play(
                current_rot,
                current_rot + 360.0,
                0.5,
                Easing::EaseInOut,
            );

            // Queue blend animation (toggle 0 <-> 1)
            let current_blend = *self.theme_blend_animation.current();
            let target_blend = if current_blend < 0.5 { 1.0 } else { 0.0 };
            self.theme_blend_animation.play(
                current_blend,
                target_blend,
                0.35,
                Easing::EaseInOut,
            );

            if let Some(ref cb) = self.on_theme_toggle {
                cb();
            }
        }

        // Update animations
        self.theme_rotation_animation.update(dt);
        self.theme_blend_animation.update(dt);
        self.theme_rotation = *self.theme_rotation_animation.current();
        self.theme_blend = *self.theme_blend_animation.current();

        self.base.request_visual_repaint();
    }

    fn draw(&self, renderer: &mut Renderer) {
        let p = &self.base.primitive;
        let frame = primitive_frame(p);
        let opacity = p.opacity;
        let palette = theme::current_theme_colors(&theme::dark_theme(), true);
        let rounding = p.rounding;

        // Shell panel background with shadow
        let style = crate::ui::primitive::make_style(p);
        renderer.draw_rect(frame.x, frame.y, frame.width, frame.height, &style);

        // Brand text
        let brand_x = frame.x + 16.0;
        let brand_y = frame.y + 20.0;
        let brand_primary_color = palette.primary.apply_opacity(opacity);
        renderer.draw_text(&self.brand_primary, brand_x, brand_y, &brand_primary_color, 22.0 / 24.0);

        let brand_sub_y = brand_y + 26.0;
        let brand_secondary_color = palette.text.with_alpha(0.5).apply_opacity(opacity);
        renderer.draw_text(&self.brand_secondary, brand_x, brand_sub_y, &brand_secondary_color, 13.0 / 24.0);

        // Selection indicator (animated)
        if !self.items.is_empty() {
            let item_height = 44.0;
            let top_offset = 80.0;
            let ind_y = frame.y + top_offset + self.selection_anim * item_height;
            let ind_x = frame.x + 8.0;
            let ind_w = frame.width - 16.0;
            let ind_color = palette.primary.with_alpha(0.15).apply_opacity(opacity);
            renderer.draw_rect_simple(ind_x, ind_y, ind_w, item_height, &ind_color, 10.0);
        }

        // Items with icon + label
        for (i, item) in self.items.iter().enumerate() {
            let rect = self.item_rect(i);
            let hover_t = if i < self.item_hover.len() { self.item_hover[i] } else { 0.0 };

            if hover_t > 0.01 {
                let hover_color = palette.surface_hover.with_alpha(0.3 * hover_t).apply_opacity(opacity);
                renderer.draw_rect_simple(rect.x, rect.y, rect.width, rect.height, &hover_color, 10.0);
            }

            // Icon (draw as text)
            let icon_x = rect.x + 12.0;
            let icon_y = rect.y + (rect.height - 20.0) * 0.5;
            let is_selected = i as i32 == self.selected_index;
            let icon_color = if is_selected {
                palette.primary.apply_opacity(opacity)
            } else {
                palette.text.with_alpha(0.6).apply_opacity(opacity)
            };
            renderer.draw_text(&item.icon, icon_x, icon_y, &icon_color, 20.0 / 24.0);

            // Label
            let label_x = rect.x + 44.0;
            let label_y = rect.y + (rect.height - 16.0) * 0.5;
            let label_color = if is_selected {
                palette.text.apply_opacity(opacity)
            } else {
                palette.text.with_alpha(0.7).apply_opacity(opacity)
            };
            renderer.draw_text(&item.label, label_x, label_y, &label_color, 16.0 / 24.0);
        }

        // Theme toggle button
        let toggle_rect = self.theme_toggle_rect();
        let toggle_bg = lerp_color(
            &palette.surface_hover,
            &palette.surface_active,
            self.theme_hover,
        ).apply_opacity(opacity);
        renderer.draw_rect_simple(toggle_rect.x, toggle_rect.y, toggle_rect.width, toggle_rect.height, &toggle_bg, toggle_rect.height * 0.5);

        // Moon/Sun icon
        let moon_alpha = (1.0 - self.theme_blend).clamp(0.0, 1.0);
        let sun_alpha = self.theme_blend.clamp(0.0, 1.0);
        let icon_x = toggle_rect.x + (toggle_rect.width - 18.0) * 0.5;
        let icon_y = toggle_rect.y + (toggle_rect.height - 18.0) * 0.5;

        if moon_alpha > 0.01 {
            let moon_color = palette.text.with_alpha(0.8 * moon_alpha).apply_opacity(opacity);
            renderer.draw_text("M", icon_x, icon_y, &moon_color, 18.0 / 24.0);
        }
        if sun_alpha > 0.01 {
            let sun_color = Color::new(1.0, 0.85, 0.2, 0.8 * sun_alpha).apply_opacity(opacity);
            renderer.draw_text("S", icon_x, icon_y, &sun_color, 18.0 / 24.0);
        }
    }

    fn reset_defaults(&mut self) {
        self.base.primitive.width = 200.0;
        self.base.primitive.height = 300.0;
        self.base.primitive.rounding = 20.0;
        self.base.primitive.border_width = 1.0;
        self.base.primitive.border_color = Color::new(0.3, 0.3, 0.3, 0.5);
        self.base.primitive.shadow = UIShadow {
            blur: 12.0,
            offset_x: 0.0,
            offset_y: 4.0,
            color: Color::new(0.0, 0.0, 0.0, 0.15),
        };
    }

    fn wants_continuous_update(&self) -> bool {
        self.theme_rotation_animation.is_active()
            || self.theme_blend_animation.is_active()
            || (self.selection_anim - self.selected_index as f32).abs() > 0.01
            || self.item_hover.iter().any(|a| *a > 0.01)
            || self.theme_hover > 0.01
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
