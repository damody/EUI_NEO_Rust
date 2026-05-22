use std::any::Any;

use crate::color::{lerp_color, Color};
use crate::rect::RectFrame;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::theme;
use crate::types::RenderLayer;
use crate::ui::node::{NodeBase, UINode};
use crate::ui::primitive::{UIPrimitive, primitive_frame};

/// Inline f32 lerp helper.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// ComboBoxNode -- dropdown selection field with popup list.
pub struct ComboBoxNode {
    pub base: NodeBase,

    // --- configuration ---
    pub items: Vec<String>,
    pub placeholder: String,
    pub selected_index: i32,
    pub font_size: f32,
    pub on_change: Option<Box<dyn Fn(i32, &str)>>,

    // --- runtime ---
    pub is_open: bool,
    pub open_state_initialized: bool,
    pub popup_presentation: bool,
    pub hover_anim: f32,
    pub open_anim: f32,
    pub item_hover_anims: Vec<f32>,
}

impl ComboBoxNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        base.primitive.width = 220.0;
        base.primitive.height = 36.0;
        Self {
            base,
            items: Vec::new(),
            placeholder: String::new(),
            selected_index: -1,
            font_size: 20.0,
            on_change: None,
            is_open: false,
            open_state_initialized: false,
            popup_presentation: false,
            hover_anim: 0.0,
            open_anim: 0.0,
            item_hover_anims: Vec::new(),
        }
    }

    /// Compute the popup list frame.
    fn popup_frame(&self) -> RectFrame {
        let frame = primitive_frame(&self.base.primitive);
        let item_height = self.font_size + 12.0;
        let popup_height = item_height * self.items.len() as f32;
        let field_vis = {
            let palette = theme::current_theme_colors(&theme::dark_theme(), true);
            theme::current_field_visuals(&palette)
        };
        RectFrame::new(
            frame.x,
            frame.y + frame.height - field_vis.popup_overlap,
            frame.width,
            popup_height,
        )
    }
}

impl UINode for ComboBoxNode {
    fn type_name(&self) -> &'static str {
        "ComboBox"
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
        let hovered = frame.contains(state.mouse_x, state.mouse_y);

        // Ensure item_hover_anims matches items count
        self.item_hover_anims.resize(self.items.len(), 0.0);

        // Hover animation
        let hover_target = if hovered { 1.0 } else { 0.0 };
        self.hover_anim = lerp_f32(self.hover_anim, hover_target, dt * 10.0);

        // Open animation
        let open_target = if self.is_open { 1.0 } else { 0.0 };
        self.open_anim = lerp_f32(self.open_anim, open_target, dt * 12.0);

        // Click to toggle open/close
        if state.mouse_clicked {
            if self.is_open {
                // Check if clicking on an item
                let popup = self.popup_frame();
                if popup.contains(state.mouse_x, state.mouse_y) {
                    let item_height = self.font_size + 12.0;
                    let relative_y = state.mouse_y - popup.y;
                    let clicked_index = (relative_y / item_height) as i32;
                    if clicked_index >= 0 && (clicked_index as usize) < self.items.len() {
                        self.selected_index = clicked_index;
                        if let Some(ref cb) = self.on_change {
                            cb(clicked_index, &self.items[clicked_index as usize]);
                        }
                    }
                    self.is_open = false;
                } else if hovered {
                    // Clicking on the field toggles
                    self.is_open = false;
                } else {
                    // Clicking outside closes
                    self.is_open = false;
                }
            } else if hovered {
                self.is_open = true;
                self.open_state_initialized = true;
            }
        }

        // Item hover animations
        if self.is_open {
            let popup = self.popup_frame();
            let item_height = self.font_size + 12.0;
            for (i, anim) in self.item_hover_anims.iter_mut().enumerate() {
                let item_rect = RectFrame::new(
                    popup.x,
                    popup.y + i as f32 * item_height,
                    popup.width,
                    item_height,
                );
                let item_hovered = item_rect.contains(state.mouse_x, state.mouse_y);
                let target = if item_hovered { 1.0 } else { 0.0 };
                *anim = lerp_f32(*anim, target, dt * 10.0);
            }
        } else {
            for anim in self.item_hover_anims.iter_mut() {
                *anim = lerp_f32(*anim, 0.0, dt * 10.0);
            }
        }

        // Popup presentation flag changes render layer
        if self.popup_presentation {
            self.base.primitive.render_layer = RenderLayer::Popup;
        }

        self.base.request_visual_repaint();
    }

    fn draw(&self, renderer: &mut Renderer) {
        let p = &self.base.primitive;
        let frame = primitive_frame(p);
        let opacity = p.opacity;
        let palette = theme::current_theme_colors(&theme::dark_theme(), true);
        let field_vis = theme::current_field_visuals(&palette);

        let rounding = field_vis.rounding;
        let inset = field_vis.horizontal_inset;
        let text_scale = self.font_size / 24.0;

        // --- Popup list ---
        if self.open_anim > 0.01 {
            let popup = self.popup_frame();
            let popup_bg = palette.surface.apply_opacity(opacity * self.open_anim);
            renderer.draw_rect_simple(popup.x, popup.y, popup.width, popup.height, &popup_bg, field_vis.popup_rounding);

            let item_height = self.font_size + 12.0;
            for (i, item) in self.items.iter().enumerate() {
                let item_y = popup.y + i as f32 * item_height;

                let hover_t = if i < self.item_hover_anims.len() { self.item_hover_anims[i] } else { 0.0 };
                if hover_t > 0.01 {
                    let highlight = palette.surface_hover.apply_opacity(opacity * hover_t * self.open_anim);
                    renderer.draw_rect_simple(popup.x, item_y, popup.width, item_height, &highlight, 4.0);
                }

                let text_color = palette.text.apply_opacity(opacity * self.open_anim);
                let text_x = popup.x + inset;
                let text_y = item_y + (item_height - self.font_size) * 0.5;
                renderer.draw_text(item, text_x, text_y, &text_color, text_scale);
            }
        }

        // --- Field chrome ---
        let bg_color = lerp_color(&palette.surface, &palette.surface_hover, self.hover_anim).apply_opacity(opacity);
        renderer.draw_rect_simple(frame.x, frame.y, frame.width, frame.height, &bg_color, rounding);

        // Border
        let border_color = lerp_color(&palette.border, &palette.primary, self.open_anim).apply_opacity(opacity);
        let bw = field_vis.border_line_height;
        renderer.draw_rect_simple(frame.x - bw, frame.y - bw, frame.width + bw * 2.0, frame.height + bw * 2.0, &border_color, rounding + bw);
        renderer.draw_rect_simple(frame.x, frame.y, frame.width, frame.height, &bg_color, rounding);

        // Selected text or placeholder
        let text_x = frame.x + inset;
        let text_y = frame.y + (frame.height - self.font_size) * 0.5;
        if self.selected_index >= 0 && (self.selected_index as usize) < self.items.len() {
            let text_color = palette.text.apply_opacity(opacity);
            renderer.draw_text(&self.items[self.selected_index as usize], text_x, text_y, &text_color, text_scale);
        } else {
            let placeholder_color = palette.text.with_alpha(0.4).apply_opacity(opacity);
            renderer.draw_text(&self.placeholder, text_x, text_y, &placeholder_color, text_scale);
        }

        // Arrow chevron (draw as "v" text placeholder)
        let arrow_color = palette.text.with_alpha(0.6).apply_opacity(opacity);
        let arrow_x = frame.x + frame.width - inset - 10.0;
        let arrow_y = frame.y + (frame.height - self.font_size) * 0.5;
        renderer.draw_text("v", arrow_x, arrow_y, &arrow_color, text_scale * 0.8);
    }

    fn reset_defaults(&mut self) {
        self.base.primitive.width = 220.0;
        self.base.primitive.height = 36.0;
    }

    fn wants_continuous_update(&self) -> bool {
        self.is_open
            || self.open_anim > 0.01
            || self.hover_anim > 0.01
            || self.item_hover_anims.iter().any(|a| *a > 0.01)
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
