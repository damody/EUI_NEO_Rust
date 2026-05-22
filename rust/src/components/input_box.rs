use std::any::Any;

use crate::color::{lerp_color, Color};
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

/// GLFW key constants (u32).
const GLFW_KEY_BACKSPACE: u32 = 259;
const GLFW_KEY_ENTER: u32 = 257;

/// InputBoxNode -- single-line text input field.
pub struct InputBoxNode {
    pub base: NodeBase,

    // --- configuration ---
    pub placeholder: String,
    pub text: String,
    pub font_size: f32,
    pub on_change: Option<Box<dyn Fn(&str)>>,
    pub on_enter: Option<Box<dyn Fn(&str)>>,

    // --- runtime ---
    pub is_focused: bool,
    pub cursor_blink_time: f32,
    pub cursor_position: usize,
    pub cursor_visible: bool,
    pub hover_anim: f32,
    pub focus_anim: f32,
}

impl InputBoxNode {
    pub fn new(key: String) -> Self {
        let mut base = NodeBase::new(key);
        base.primitive.width = 220.0;
        base.primitive.height = 36.0;
        Self {
            base,
            placeholder: String::new(),
            text: String::new(),
            font_size: 20.0,
            on_change: None,
            on_enter: None,
            is_focused: false,
            cursor_blink_time: 0.0,
            cursor_position: 0,
            cursor_visible: true,
            hover_anim: 0.0,
            focus_anim: 0.0,
        }
    }

    /// Clamp cursor position to valid range within the text (char boundary aware).
    fn clamp_cursor(&mut self) {
        let char_count = self.text.chars().count();
        if self.cursor_position > char_count {
            self.cursor_position = char_count;
        }
    }

    /// Delete the character before the cursor (UTF-8 aware).
    fn backspace(&mut self) {
        if self.cursor_position == 0 {
            return;
        }
        // Find byte offset of cursor_position - 1 and cursor_position
        let mut chars = self.text.char_indices();
        let mut byte_start = 0;
        let mut byte_end = self.text.len();
        let mut idx = 0;
        for (byte_offset, ch) in &mut chars {
            if idx == self.cursor_position - 1 {
                byte_start = byte_offset;
                byte_end = byte_offset + ch.len_utf8();
                break;
            }
            idx += 1;
        }
        self.text.replace_range(byte_start..byte_end, "");
        self.cursor_position -= 1;
    }

    /// Insert text at current cursor position (UTF-8 aware).
    fn insert_text(&mut self, input: &str) {
        if input.is_empty() {
            return;
        }
        // Find byte offset for cursor_position
        let byte_offset = self
            .text
            .char_indices()
            .nth(self.cursor_position)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        self.text.insert_str(byte_offset, input);
        self.cursor_position += input.chars().count();
    }
}

impl UINode for InputBoxNode {
    fn type_name(&self) -> &'static str {
        "InputBox"
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

        // Focus management: click inside -> focus, click outside -> unfocus
        if state.mouse_clicked {
            self.is_focused = hovered;
            if self.is_focused {
                // Place cursor at end on fresh focus
                self.cursor_position = self.text.chars().count();
                self.cursor_blink_time = 0.0;
                self.cursor_visible = true;
            }
        }

        // Hover animation
        let hover_target = if hovered { 1.0 } else { 0.0 };
        self.hover_anim = lerp_f32(self.hover_anim, hover_target, dt * 10.0);

        // Focus animation
        let focus_target = if self.is_focused { 1.0 } else { 0.0 };
        self.focus_anim = lerp_f32(self.focus_anim, focus_target, dt * 10.0);

        if self.is_focused {
            // Cursor blink (0.5s on, 0.5s off)
            self.cursor_blink_time += dt;
            if self.cursor_blink_time >= 1.0 {
                self.cursor_blink_time -= 1.0;
            }
            self.cursor_visible = self.cursor_blink_time < 0.5;

            // Text input
            if !state.text_input.is_empty() {
                self.insert_text(&state.text_input.clone());
                self.cursor_blink_time = 0.0;
                self.cursor_visible = true;
                if let Some(ref cb) = self.on_change {
                    cb(&self.text);
                }
            }

            // Backspace key
            if (GLFW_KEY_BACKSPACE as usize) < state.keys_pressed.len()
                && state.keys_pressed[GLFW_KEY_BACKSPACE as usize]
            {
                self.backspace();
                self.cursor_blink_time = 0.0;
                self.cursor_visible = true;
                if let Some(ref cb) = self.on_change {
                    cb(&self.text);
                }
            }

            // Enter key
            if (GLFW_KEY_ENTER as usize) < state.keys_pressed.len()
                && state.keys_pressed[GLFW_KEY_ENTER as usize]
            {
                if let Some(ref cb) = self.on_enter {
                    cb(&self.text);
                }
            }
        }

        self.clamp_cursor();
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

        // Field background
        let bg_color = lerp_color(
            &palette.surface,
            &palette.surface_hover,
            self.hover_anim,
        )
        .apply_opacity(opacity);
        renderer.draw_rect_simple(frame.x, frame.y, frame.width, frame.height, &bg_color, rounding);

        // Border
        let border_color = lerp_color(
            &palette.border,
            &palette.primary,
            self.focus_anim,
        )
        .apply_opacity(opacity);
        let bw = field_vis.border_line_height;
        renderer.draw_rect_simple(frame.x - bw, frame.y - bw, frame.width + bw * 2.0, frame.height + bw * 2.0, &border_color, rounding + bw);
        renderer.draw_rect_simple(frame.x, frame.y, frame.width, frame.height, &bg_color, rounding);

        // Focus line at the bottom
        if self.focus_anim > 0.01 {
            let focus_color = palette.primary.apply_opacity(opacity * self.focus_anim);
            renderer.draw_rect_simple(
                frame.x,
                frame.y + frame.height - field_vis.focus_line_height,
                frame.width,
                field_vis.focus_line_height,
                &focus_color,
                0.0,
            );
        }

        // Text or placeholder
        let text_scale = self.font_size / 24.0;
        let text_y = frame.y + (frame.height - self.font_size) * 0.5;
        let text_x = frame.x + inset;

        if self.text.is_empty() {
            let placeholder_color = palette.text.with_alpha(0.4).apply_opacity(opacity);
            renderer.draw_text(&self.placeholder, text_x, text_y, &placeholder_color, text_scale);
        } else {
            let text_color = palette.text.apply_opacity(opacity);
            renderer.draw_text(&self.text, text_x, text_y, &text_color, text_scale);
        }

        // Cursor bar
        if self.is_focused && self.cursor_visible {
            let cursor_text = &self.text[..self.text.char_indices().nth(self.cursor_position).map(|(i,_)| i).unwrap_or(self.text.len())];
            let cursor_x = text_x + renderer.measure_text_width(cursor_text, text_scale);
            let cursor_h = self.font_size + 2.0;
            let cursor_y = frame.y + (frame.height - cursor_h) * 0.5;
            let cursor_color = palette.primary.apply_opacity(opacity);
            renderer.draw_rect_simple(cursor_x, cursor_y, 1.5, cursor_h, &cursor_color, 0.0);
        }
    }

    fn reset_defaults(&mut self) {
        self.base.primitive.width = 220.0;
        self.base.primitive.height = 36.0;
    }

    fn wants_continuous_update(&self) -> bool {
        self.is_focused || self.hover_anim > 0.01 || self.focus_anim > 0.01
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
