use std::ffi::CString;
use std::ptr;

use crate::color::Color;
use crate::rect::{RectFrame, RectBounds, Point2};
use crate::types::{RectStyle, RectGradient};
use crate::debug::dump::DumpState;
use super::shaders;
use super::font::FontAtlas;

/// SDF constants matching C++ kTextSdf* values.
const TEXT_SDF_PADDING: i32 = 5;
const TEXT_SDF_ON_EDGE_VALUE: f32 = 128.0;

/// OpenGL 3.3 renderer (translates C++ static Renderer methods to struct).
pub struct Renderer {
    // Shader programs
    rect_program: u32,
    text_program: u32,
    polygon_program: u32,
    cached_blur_program: u32,
    composite_program: u32,
    // VAOs/VBOs
    rect_vao: u32,
    rect_vbo: u32,
    text_vao: u32,
    text_vbo: u32,
    polygon_vao: u32,
    polygon_vbo: u32,
    composite_vao: u32,
    composite_vbo: u32,
    // Uniform locations (rect shader)
    proj_loc: i32,
    color_loc: i32,
    pos_loc: i32,
    size_loc: i32,
    rounding_loc: i32,
    box_pos_loc: i32,
    box_size_loc: i32,
    translate_loc: i32,
    scale_loc: i32,
    rotation_loc: i32,
    transform_inv_loc: i32,
    blur_amount_loc: i32,
    shadow_blur_loc: i32,
    shadow_offset_loc: i32,
    shadow_color_loc: i32,
    gradient_enabled_loc: i32,
    gradient_tl_loc: i32,
    gradient_tr_loc: i32,
    gradient_bl_loc: i32,
    gradient_br_loc: i32,
    time_loc: i32,
    resolution_loc: i32,
    channel0_loc: i32,
    // Text shader uniform locations
    text_proj_loc: i32,
    text_color_loc: i32,
    text_mode_loc: i32,
    text_sdf_edge_loc: i32,
    text_sdf_px_range_loc: i32,
    // Cached blur uniform locations
    cached_blur_proj_loc: i32,
    cached_blur_pos_loc: i32,
    cached_blur_size_loc: i32,
    cached_blur_texture_loc: i32,
    cached_blur_box_pos_loc: i32,
    cached_blur_box_size_loc: i32,
    cached_blur_translate_loc: i32,
    cached_blur_transform_inv_loc: i32,
    cached_blur_rounding_loc: i32,
    cached_blur_shadow_blur_loc: i32,
    cached_blur_shadow_offset_loc: i32,
    cached_blur_shadow_alpha_loc: i32,
    // Composite uniform locations
    composite_proj_loc: i32,
    composite_pos_loc: i32,
    composite_size_loc: i32,
    composite_texture_loc: i32,
    composite_uv_pos_loc: i32,
    composite_uv_size_loc: i32,
    // Polygon uniform locations
    polygon_proj_loc: i32,
    polygon_color_loc: i32,
    polygon_gradient_enabled_loc: i32,
    polygon_gradient_tl_loc: i32,
    polygon_gradient_tr_loc: i32,
    polygon_gradient_bl_loc: i32,
    polygon_gradient_br_loc: i32,
    // Background texture
    bg_texture: u32,
    // Polygon VBO capacity tracking
    polygon_vbo_capacity: isize,
    // Current active program (to avoid redundant glUseProgram calls)
    current_active_program: u32,
    // Font atlas
    pub font_atlas: FontAtlas,
    // Dump state (mirrors C++ static DumpState)
    pub dump: DumpState,
    // State
    initialized: bool,
    needs_repaint: bool,
    animation_time_left: f32,
    viewport_width: f32,
    viewport_height: f32,
    framebuffer_width: f32,
    framebuffer_height: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            rect_program: 0,
            text_program: 0,
            polygon_program: 0,
            cached_blur_program: 0,
            composite_program: 0,
            rect_vao: 0,
            rect_vbo: 0,
            text_vao: 0,
            text_vbo: 0,
            polygon_vao: 0,
            polygon_vbo: 0,
            composite_vao: 0,
            composite_vbo: 0,
            proj_loc: -1,
            color_loc: -1,
            pos_loc: -1,
            size_loc: -1,
            rounding_loc: -1,
            box_pos_loc: -1,
            box_size_loc: -1,
            translate_loc: -1,
            scale_loc: -1,
            rotation_loc: -1,
            transform_inv_loc: -1,
            blur_amount_loc: -1,
            shadow_blur_loc: -1,
            shadow_offset_loc: -1,
            shadow_color_loc: -1,
            gradient_enabled_loc: -1,
            gradient_tl_loc: -1,
            gradient_tr_loc: -1,
            gradient_bl_loc: -1,
            gradient_br_loc: -1,
            time_loc: -1,
            resolution_loc: -1,
            channel0_loc: -1,
            text_proj_loc: -1,
            text_color_loc: -1,
            text_mode_loc: -1,
            text_sdf_edge_loc: -1,
            text_sdf_px_range_loc: -1,
            cached_blur_proj_loc: -1,
            cached_blur_pos_loc: -1,
            cached_blur_size_loc: -1,
            cached_blur_texture_loc: -1,
            cached_blur_box_pos_loc: -1,
            cached_blur_box_size_loc: -1,
            cached_blur_translate_loc: -1,
            cached_blur_transform_inv_loc: -1,
            cached_blur_rounding_loc: -1,
            cached_blur_shadow_blur_loc: -1,
            cached_blur_shadow_offset_loc: -1,
            cached_blur_shadow_alpha_loc: -1,
            composite_proj_loc: -1,
            composite_pos_loc: -1,
            composite_size_loc: -1,
            composite_texture_loc: -1,
            composite_uv_pos_loc: -1,
            composite_uv_size_loc: -1,
            polygon_proj_loc: -1,
            polygon_color_loc: -1,
            polygon_gradient_enabled_loc: -1,
            polygon_gradient_tl_loc: -1,
            polygon_gradient_tr_loc: -1,
            polygon_gradient_bl_loc: -1,
            polygon_gradient_br_loc: -1,
            bg_texture: 0,
            polygon_vbo_capacity: 0,
            current_active_program: 0,
            font_atlas: FontAtlas::new(),
            dump: DumpState::new(),
            initialized: false,
            needs_repaint: true,
            animation_time_left: 0.0,
            viewport_width: 0.0,
            viewport_height: 0.0,
            framebuffer_width: 0.0,
            framebuffer_height: 0.0,
        }
    }

    /// Compile shaders, create VAOs/VBOs, cache uniform locations.
    pub fn init(&mut self) {
        unsafe {
            // ── Compile shader programs ────────────────────────────────
            self.rect_program = create_program(shaders::RECT_VERTEX_SHADER, shaders::RECT_FRAGMENT_SHADER);
            self.cached_blur_program = create_program(shaders::CACHED_BLUR_VERTEX_SHADER, shaders::CACHED_BLUR_FRAGMENT_SHADER);
            self.composite_program = create_program(shaders::COMPOSITE_VERTEX_SHADER, shaders::COMPOSITE_FRAGMENT_SHADER);
            self.polygon_program = create_program(shaders::POLYGON_VERTEX_SHADER, shaders::POLYGON_FRAGMENT_SHADER);
            self.text_program = create_program(shaders::TEXT_VERTEX_SHADER, shaders::TEXT_FRAGMENT_SHADER);

            // ── Rect shader uniforms ───────────────────────────────────
            self.proj_loc = get_uniform(self.rect_program, "projection");
            self.color_loc = get_uniform(self.rect_program, "uColor");
            self.pos_loc = get_uniform(self.rect_program, "uPos");
            self.size_loc = get_uniform(self.rect_program, "uSize");
            self.box_pos_loc = get_uniform(self.rect_program, "uBoxPos");
            self.box_size_loc = get_uniform(self.rect_program, "uBoxSize");
            self.translate_loc = get_uniform(self.rect_program, "uTranslate");
            self.scale_loc = get_uniform(self.rect_program, "uScale");
            self.rotation_loc = get_uniform(self.rect_program, "uRotation");
            self.transform_inv_loc = get_uniform(self.rect_program, "uTransformInv");
            self.rounding_loc = get_uniform(self.rect_program, "uRounding");
            self.blur_amount_loc = get_uniform(self.rect_program, "uBlurAmount");
            self.shadow_blur_loc = get_uniform(self.rect_program, "uShadowBlur");
            self.shadow_offset_loc = get_uniform(self.rect_program, "uShadowOffset");
            self.shadow_color_loc = get_uniform(self.rect_program, "uShadowColor");
            self.gradient_enabled_loc = get_uniform(self.rect_program, "uGradientEnabled");
            self.gradient_tl_loc = get_uniform(self.rect_program, "uGradientTopLeft");
            self.gradient_tr_loc = get_uniform(self.rect_program, "uGradientTopRight");
            self.gradient_bl_loc = get_uniform(self.rect_program, "uGradientBottomLeft");
            self.gradient_br_loc = get_uniform(self.rect_program, "uGradientBottomRight");
            self.time_loc = get_uniform(self.rect_program, "iTime");
            self.resolution_loc = get_uniform(self.rect_program, "iResolution");
            self.channel0_loc = get_uniform(self.rect_program, "iChannel0");

            // ── Cached blur shader uniforms ────────────────────────────
            self.cached_blur_proj_loc = get_uniform(self.cached_blur_program, "projection");
            self.cached_blur_pos_loc = get_uniform(self.cached_blur_program, "uPos");
            self.cached_blur_size_loc = get_uniform(self.cached_blur_program, "uSize");
            self.cached_blur_texture_loc = get_uniform(self.cached_blur_program, "uTexture");
            self.cached_blur_box_pos_loc = get_uniform(self.cached_blur_program, "uBoxPos");
            self.cached_blur_box_size_loc = get_uniform(self.cached_blur_program, "uBoxSize");
            self.cached_blur_translate_loc = get_uniform(self.cached_blur_program, "uTranslate");
            self.cached_blur_transform_inv_loc = get_uniform(self.cached_blur_program, "uTransformInv");
            self.cached_blur_rounding_loc = get_uniform(self.cached_blur_program, "uRounding");
            self.cached_blur_shadow_blur_loc = get_uniform(self.cached_blur_program, "uShadowBlur");
            self.cached_blur_shadow_offset_loc = get_uniform(self.cached_blur_program, "uShadowOffset");
            self.cached_blur_shadow_alpha_loc = get_uniform(self.cached_blur_program, "uShadowAlpha");

            // ── Composite shader uniforms ──────────────────────────────
            self.composite_proj_loc = get_uniform(self.composite_program, "projection");
            self.composite_pos_loc = get_uniform(self.composite_program, "uPos");
            self.composite_size_loc = get_uniform(self.composite_program, "uSize");
            self.composite_texture_loc = get_uniform(self.composite_program, "uTexture");
            self.composite_uv_pos_loc = get_uniform(self.composite_program, "uUVPos");
            self.composite_uv_size_loc = get_uniform(self.composite_program, "uUVSize");

            // ── Polygon shader uniforms ────────────────────────────────
            self.polygon_proj_loc = get_uniform(self.polygon_program, "projection");
            self.polygon_color_loc = get_uniform(self.polygon_program, "uColor");
            self.polygon_gradient_enabled_loc = get_uniform(self.polygon_program, "uGradientEnabled");
            self.polygon_gradient_tl_loc = get_uniform(self.polygon_program, "uGradientTopLeft");
            self.polygon_gradient_tr_loc = get_uniform(self.polygon_program, "uGradientTopRight");
            self.polygon_gradient_bl_loc = get_uniform(self.polygon_program, "uGradientBottomLeft");
            self.polygon_gradient_br_loc = get_uniform(self.polygon_program, "uGradientBottomRight");

            // ── Text shader uniforms ───────────────────────────────────
            self.text_proj_loc = get_uniform(self.text_program, "projection");
            self.text_color_loc = get_uniform(self.text_program, "textColor");
            self.text_mode_loc = get_uniform(self.text_program, "textMode");
            self.text_sdf_edge_loc = get_uniform(self.text_program, "sdfEdgeValue");
            self.text_sdf_px_range_loc = get_uniform(self.text_program, "sdfPxRange");

            // ── Set texture sampler defaults ───────────────────────────
            gl::UseProgram(self.rect_program);
            gl::Uniform1i(self.channel0_loc, 0);
            gl::UseProgram(self.cached_blur_program);
            gl::Uniform1i(self.cached_blur_texture_loc, 0);
            gl::UseProgram(self.composite_program);
            gl::Uniform1i(self.composite_texture_loc, 0);
            gl::UseProgram(self.text_program);
            gl::Uniform1i(get_uniform(self.text_program, "text"), 0);
            gl::Uniform1i(self.text_mode_loc, 1);
            gl::Uniform1f(self.text_sdf_edge_loc, TEXT_SDF_ON_EDGE_VALUE / 255.0);
            gl::Uniform1f(self.text_sdf_px_range_loc, TEXT_SDF_PADDING as f32);

            // ── Background texture ─────────────────────────────────────
            gl::GenTextures(1, &mut self.bg_texture);
            gl::BindTexture(gl::TEXTURE_2D, self.bg_texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            // ── Rect VAO/VBO (unit quad) ───────────────────────────────
            let vertices: [f32; 8] = [
                0.0, 0.0,
                1.0, 0.0,
                0.0, 1.0,
                1.0, 1.0,
            ];
            gl::GenVertexArrays(1, &mut self.rect_vao);
            gl::GenBuffers(1, &mut self.rect_vbo);
            gl::BindVertexArray(self.rect_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.rect_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<f32>() as i32, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            // ── Polygon VAO/VBO (dynamic) ──────────────────────────────
            gl::GenVertexArrays(1, &mut self.polygon_vao);
            gl::GenBuffers(1, &mut self.polygon_vbo);
            gl::BindVertexArray(self.polygon_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.polygon_vbo);
            self.polygon_vbo_capacity = (std::mem::size_of::<f32>() * 4 * 3) as isize;
            gl::BufferData(gl::ARRAY_BUFFER, self.polygon_vbo_capacity, ptr::null(), gl::DYNAMIC_DRAW);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, (2 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            // ── Text VAO/VBO (dynamic, 6 vertices × 4 floats) ─────────
            gl::GenVertexArrays(1, &mut self.text_vao);
            gl::GenBuffers(1, &mut self.text_vbo);
            gl::BindVertexArray(self.text_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.text_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * 6 * 4) as isize,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            // ── Composite VAO/VBO (6 verts with UV) ────────────────────
            let composite_vertices: [f32; 24] = [
                0.0, 0.0, 0.0, 1.0,
                1.0, 0.0, 1.0, 1.0,
                1.0, 1.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
                1.0, 1.0, 1.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
            ];
            gl::GenVertexArrays(1, &mut self.composite_vao);
            gl::GenBuffers(1, &mut self.composite_vbo);
            gl::BindVertexArray(self.composite_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.composite_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (composite_vertices.len() * std::mem::size_of::<f32>()) as isize,
                composite_vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<f32>() as i32, (2 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        self.initialized = true;
        eprintln!("[Renderer] Initialized (rect={}, text={}, polygon={}, blur={}, composite={})",
            self.rect_program, self.text_program, self.polygon_program,
            self.cached_blur_program, self.composite_program);
    }

    /// Delete all GL resources.
    pub fn shutdown(&mut self) {
        self.font_atlas.destroy();
        unsafe {
            gl::DeleteVertexArrays(1, &self.rect_vao);
            gl::DeleteBuffers(1, &self.rect_vbo);
            gl::DeleteVertexArrays(1, &self.text_vao);
            gl::DeleteBuffers(1, &self.text_vbo);
            gl::DeleteVertexArrays(1, &self.polygon_vao);
            gl::DeleteBuffers(1, &self.polygon_vbo);
            gl::DeleteVertexArrays(1, &self.composite_vao);
            gl::DeleteBuffers(1, &self.composite_vbo);
            gl::DeleteProgram(self.rect_program);
            gl::DeleteProgram(self.text_program);
            gl::DeleteProgram(self.polygon_program);
            gl::DeleteProgram(self.cached_blur_program);
            gl::DeleteProgram(self.composite_program);
            gl::DeleteTextures(1, &self.bg_texture);
        }
        self.initialized = false;
    }

    /// Returns true if the renderer has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set the viewport dimensions (call on resize).
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Set the framebuffer dimensions (for resolution uniform).
    pub fn set_framebuffer_size(&mut self, width: f32, height: f32) {
        self.framebuffer_width = width;
        self.framebuffer_height = height;
    }

    /// Begin a new frame: set viewport, clear buffers, update projection.
    pub fn begin_frame(&mut self) {
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);

            // Y-down orthographic projection: L=0, R=screenW, T=0, B=screenH
            let l = 0.0f32;
            let r = self.viewport_width;
            let t = 0.0f32;
            let b = self.viewport_height;

            let proj: [f32; 16] = [
                2.0 / (r - l),        0.0,                 0.0, 0.0,
                0.0,                   2.0 / (t - b),       0.0, 0.0,
                0.0,                   0.0,                -1.0, 0.0,
                -(r + l) / (r - l),   -(t + b) / (t - b),  0.0, 1.0,
            ];

            // Upload projection to all shader programs
            gl::UseProgram(self.rect_program);
            gl::UniformMatrix4fv(self.proj_loc, 1, gl::FALSE, proj.as_ptr());
            gl::Uniform1f(self.time_loc, 0.0); // TODO: pass actual time if needed
            gl::Uniform2f(self.resolution_loc, self.framebuffer_width, self.framebuffer_height);

            gl::UseProgram(self.cached_blur_program);
            gl::UniformMatrix4fv(self.cached_blur_proj_loc, 1, gl::FALSE, proj.as_ptr());

            gl::UseProgram(self.polygon_program);
            gl::UniformMatrix4fv(self.polygon_proj_loc, 1, gl::FALSE, proj.as_ptr());

            gl::UseProgram(self.text_program);
            gl::UniformMatrix4fv(self.text_proj_loc, 1, gl::FALSE, proj.as_ptr());

            gl::UseProgram(self.composite_program);
            gl::UniformMatrix4fv(self.composite_proj_loc, 1, gl::FALSE, proj.as_ptr());

            self.current_active_program = self.text_program;

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);

            // Clear to dark background
            gl::ClearColor(0.11, 0.11, 0.12, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    /// Draw a rounded rectangle with full style (color, gradient, shadow, blur, transform).
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, style: &RectStyle) {
        if self.dump.is_recording() {
            self.dump.record_rect(x, y, w, h, style);
        }
        let bounds = self.measure_rect_bounds(x, y, w, h, style);

        unsafe {
            if self.current_active_program != self.rect_program {
                gl::UseProgram(self.rect_program);
                self.current_active_program = self.rect_program;
            }

            let transform_inv = build_transform_inverse(style.transform.rotation_degrees);

            gl::Uniform2f(self.pos_loc, bounds.x, bounds.y);
            gl::Uniform2f(self.size_loc, bounds.w, bounds.h);
            gl::Uniform2f(self.box_pos_loc, x, y);
            gl::Uniform2f(self.box_size_loc, w, h);
            gl::Uniform2f(self.translate_loc, style.transform.translate_x, style.transform.translate_y);
            gl::Uniform2f(self.scale_loc, style.transform.scale_x, style.transform.scale_y);
            gl::Uniform1f(self.rotation_loc, style.transform.rotation_degrees);
            gl::UniformMatrix2fv(self.transform_inv_loc, 1, gl::FALSE, transform_inv.as_ptr());
            gl::Uniform4f(self.color_loc, style.color.r, style.color.g, style.color.b, style.color.a);
            gl::Uniform1f(self.rounding_loc, style.rounding);
            gl::Uniform1f(self.blur_amount_loc, style.blur_amount);
            gl::Uniform1f(self.shadow_blur_loc, style.shadow_blur);
            gl::Uniform2f(self.shadow_offset_loc, style.shadow_offset_x, style.shadow_offset_y);
            gl::Uniform4f(
                self.shadow_color_loc,
                style.shadow_color.r, style.shadow_color.g,
                style.shadow_color.b, style.shadow_color.a,
            );
            gl::Uniform1i(self.gradient_enabled_loc, if style.gradient.enabled { 1 } else { 0 });
            gl::Uniform4f(self.gradient_tl_loc, style.gradient.top_left.r, style.gradient.top_left.g, style.gradient.top_left.b, style.gradient.top_left.a);
            gl::Uniform4f(self.gradient_tr_loc, style.gradient.top_right.r, style.gradient.top_right.g, style.gradient.top_right.b, style.gradient.top_right.a);
            gl::Uniform4f(self.gradient_bl_loc, style.gradient.bottom_left.r, style.gradient.bottom_left.g, style.gradient.bottom_left.b, style.gradient.bottom_left.a);
            gl::Uniform4f(self.gradient_br_loc, style.gradient.bottom_right.r, style.gradient.bottom_right.g, style.gradient.bottom_right.b, style.gradient.bottom_right.a);

            if style.blur_amount > 0.0 {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.bg_texture);
            }

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::BindVertexArray(self.rect_vao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    /// Convenience: draw a rect with a solid color and rounding, no other effects.
    pub fn draw_rect_simple(&mut self, x: f32, y: f32, w: f32, h: f32, color: &Color, rounding: f32) {
        let style = RectStyle {
            color: *color,
            rounding,
            ..Default::default()
        };
        self.draw_rect(x, y, w, h, &style);
    }

    /// Draw a filled polygon with optional gradient and stroke.
    pub fn draw_polygon(
        &mut self,
        points: &[Point2],
        fill_color: &Color,
        gradient: &RectGradient,
        stroke_width: f32,
        stroke_color: &Color,
    ) {
        if self.dump.is_recording() {
            self.dump.record_polygon(points, fill_color, gradient, stroke_width, stroke_color);
        }
        if points.len() < 3 {
            return;
        }

        let triangles = triangulate_polygon(points);
        if triangles.is_empty() {
            return;
        }

        // Compute bounding box for UV mapping
        let fill_bounds = compute_polygon_bounds(points);
        let inv_w = if fill_bounds.w > 0.0001 { 1.0 / fill_bounds.w } else { 0.0 };
        let inv_h = if fill_bounds.h > 0.0001 { 1.0 / fill_bounds.h } else { 0.0 };

        let mut triangle_vertices: Vec<f32> = Vec::with_capacity(triangles.len() * 4);
        for p in &triangles {
            triangle_vertices.push(p.x);
            triangle_vertices.push(p.y);
            triangle_vertices.push((p.x - fill_bounds.x) * inv_w);
            triangle_vertices.push((p.y - fill_bounds.y) * inv_h);
        }

        unsafe {
            if self.current_active_program != self.polygon_program {
                gl::UseProgram(self.polygon_program);
                self.current_active_program = self.polygon_program;
            }

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Uniform4f(self.polygon_color_loc, fill_color.r, fill_color.g, fill_color.b, fill_color.a);
            gl::Uniform1i(self.polygon_gradient_enabled_loc, if gradient.enabled { 1 } else { 0 });
            gl::Uniform4f(self.polygon_gradient_tl_loc, gradient.top_left.r, gradient.top_left.g, gradient.top_left.b, gradient.top_left.a);
            gl::Uniform4f(self.polygon_gradient_tr_loc, gradient.top_right.r, gradient.top_right.g, gradient.top_right.b, gradient.top_right.a);
            gl::Uniform4f(self.polygon_gradient_bl_loc, gradient.bottom_left.r, gradient.bottom_left.g, gradient.bottom_left.b, gradient.bottom_left.a);
            gl::Uniform4f(self.polygon_gradient_br_loc, gradient.bottom_right.r, gradient.bottom_right.g, gradient.bottom_right.b, gradient.bottom_right.a);

            gl::BindVertexArray(self.polygon_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.polygon_vbo);
            let triangle_bytes = (std::mem::size_of::<f32>() * triangle_vertices.len()) as isize;
            if triangle_bytes > self.polygon_vbo_capacity {
                self.polygon_vbo_capacity = triangle_bytes;
                gl::BufferData(gl::ARRAY_BUFFER, self.polygon_vbo_capacity, ptr::null(), gl::DYNAMIC_DRAW);
            }
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, triangle_bytes, triangle_vertices.as_ptr() as *const _);
            gl::DrawArrays(gl::TRIANGLES, 0, triangles.len() as i32);

            // Stroke
            if stroke_width > 0.0 && stroke_color.a > 0.0 {
                let mut outline_vertices: Vec<f32> = Vec::with_capacity(points.len() * 4);
                for p in points {
                    outline_vertices.push(p.x);
                    outline_vertices.push(p.y);
                    outline_vertices.push(0.0);
                    outline_vertices.push(0.0);
                }
                gl::Uniform4f(self.polygon_color_loc, stroke_color.r, stroke_color.g, stroke_color.b, stroke_color.a);
                gl::Uniform1i(self.polygon_gradient_enabled_loc, 0);
                let outline_bytes = (std::mem::size_of::<f32>() * outline_vertices.len()) as isize;
                if outline_bytes > self.polygon_vbo_capacity {
                    self.polygon_vbo_capacity = outline_bytes;
                    gl::BufferData(gl::ARRAY_BUFFER, self.polygon_vbo_capacity, ptr::null(), gl::DYNAMIC_DRAW);
                }
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, outline_bytes, outline_vertices.as_ptr() as *const _);
                gl::LineWidth(stroke_width.max(1.0));
                gl::DrawArrays(gl::LINE_LOOP, 0, points.len() as i32);
                gl::LineWidth(1.0);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }

    /// Draw text at (x, y) with the given color and normalized scale.
    /// Scale convention matches C++: `scale = fontSize / 24.0`.
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, color: &Color, scale: f32) {
        if self.dump.is_recording() {
            self.dump.record_text(text, x, y, color, scale);
        }
        if text.is_empty() {
            return;
        }

        // Matches C++ resolve_requested_text_pixel_size
        let requested_px = 24.0 * scale;

        unsafe {
            if self.current_active_program != self.text_program {
                gl::UseProgram(self.text_program);
                self.current_active_program = self.text_program;
            }

            gl::Uniform4f(self.text_color_loc, color.r, color.g, color.b, color.a);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.text_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.text_vbo);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let mut cursor_x = x.round();
            let cursor_y = y.round();
            let mut current_texture: u32 = 0;

            for ch in text.chars() {
                let codepoint = ch as u32;

                if ch == ' ' {
                    cursor_x += requested_px * 0.3;
                    continue;
                }

                if let Some(glyph) = self.font_atlas.get_glyph(codepoint) {
                    let g_scale = requested_px / glyph.base_pixel_size.max(1.0);

                    let xpos = cursor_x + glyph.render_bearing_x * g_scale;
                    let ypos = cursor_y + glyph.render_bearing_y * g_scale;
                    let w = glyph.render_width * g_scale;
                    let h = glyph.render_height * g_scale;

                    let vertices: [f32; 24] = [
                        xpos,     ypos + h, 0.0, 1.0,
                        xpos,     ypos,     0.0, 0.0,
                        xpos + w, ypos,     1.0, 0.0,
                        xpos,     ypos + h, 0.0, 1.0,
                        xpos + w, ypos,     1.0, 0.0,
                        xpos + w, ypos + h, 1.0, 1.0,
                    ];

                    if glyph.texture_id != current_texture && glyph.texture_id != 0 {
                        gl::BindTexture(gl::TEXTURE_2D, glyph.texture_id);
                        current_texture = glyph.texture_id;
                    }
                    if glyph.texture_id != 0 {
                        gl::BufferSubData(
                            gl::ARRAY_BUFFER, 0,
                            std::mem::size_of_val(&vertices) as isize,
                            vertices.as_ptr() as *const _,
                        );
                        gl::DrawArrays(gl::TRIANGLES, 0, 6);
                    }
                    cursor_x += glyph.advance * g_scale;
                } else {
                    cursor_x += requested_px * 0.3;
                }
            }
        }
    }

    /// Measure the bounding box of rendered text (matches C++ MeasureTextBounds).
    /// Returns RectFrame with x/y as visible bearing offsets.
    pub fn measure_text_bounds(&self, text: &str, scale: f32) -> RectFrame {
        self.font_atlas.measure_text_bounds(text, scale)
    }

    /// Measure the width of rendered text (matches C++ MeasureTextWidth).
    pub fn measure_text_width(&self, text: &str, scale: f32) -> f32 {
        self.font_atlas.measure_text_width(text, scale)
    }

    /// Compute the visual bounds of a rect including shadow and blur expansion.
    pub fn measure_rect_bounds(&self, x: f32, y: f32, w: f32, h: f32, style: &RectStyle) -> RectBounds {
        let expand = style.shadow_blur.max(style.blur_amount * self.viewport_width.min(self.viewport_height));
        let sx = style.shadow_offset_x;
        let sy = style.shadow_offset_y;
        RectBounds {
            x: x - expand + sx.min(0.0),
            y: y - expand + sy.min(0.0),
            w: w + expand * 2.0 + sx.abs(),
            h: h + expand * 2.0 + sy.abs(),
        }
    }

    /// Request a repaint for the given duration (seconds).
    pub fn request_repaint(&mut self, duration: f32) {
        self.needs_repaint = true;
        if duration > self.animation_time_left {
            self.animation_time_left = duration;
        }
    }

    /// Returns true if the frame should be redrawn.
    pub fn should_repaint(&self) -> bool {
        self.needs_repaint || self.animation_time_left > 0.0
    }

    /// Tick the animation timer by delta_time seconds.
    pub fn tick(&mut self, delta_time: f32) {
        if self.animation_time_left > 0.0 {
            self.animation_time_left -= delta_time;
            if self.animation_time_left < 0.0 {
                self.animation_time_left = 0.0;
            }
        }
        self.needs_repaint = false;
    }

    /// Mark everything as needing redraw.
    pub fn invalidate_all(&mut self) {
        self.needs_repaint = true;
    }
}

// ── Helper functions ──────────────────────────────────────────────────

/// Build 2×2 rotation inverse matrix (column-major) from degrees.
fn build_transform_inverse(rotation_degrees: f32) -> [f32; 4] {
    let rad = rotation_degrees * std::f32::consts::PI / 180.0;
    let c = rad.cos();
    let s = rad.sin();
    [c, s, -s, c] // column-major: [col0.x, col0.y, col1.x, col1.y]
}

/// Compute bounding box of polygon points.
fn compute_polygon_bounds(points: &[Point2]) -> RectBounds {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    RectBounds {
        x: min_x,
        y: min_y,
        w: max_x - min_x,
        h: max_y - min_y,
    }
}

/// Ear-clipping polygon triangulation.
fn triangulate_polygon(points: &[Point2]) -> Vec<Point2> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity((n - 2) * 3);
    let mut indices: Vec<usize> = (0..n).collect();

    let mut remaining = n;
    let mut fail_count = 0;
    let mut i = 0;

    while remaining > 2 {
        if fail_count >= remaining {
            // Degenerate polygon, emit remaining as fan
            for j in 1..remaining - 1 {
                result.push(points[indices[0]]);
                result.push(points[indices[j]]);
                result.push(points[indices[j + 1]]);
            }
            break;
        }

        let prev = if i == 0 { remaining - 1 } else { i - 1 };
        let curr = i;
        let next = if i + 1 >= remaining { 0 } else { i + 1 };

        let a = points[indices[prev]];
        let b = points[indices[curr]];
        let c = points[indices[next]];

        // Check if ear (convex and no other point inside)
        let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if cross > 0.0 {
            // Check no other vertex inside triangle
            let mut ear_valid = true;
            for k in 0..remaining {
                if k == prev || k == curr || k == next {
                    continue;
                }
                if point_in_triangle(points[indices[k]], a, b, c) {
                    ear_valid = false;
                    break;
                }
            }

            if ear_valid {
                result.push(a);
                result.push(b);
                result.push(c);
                indices.remove(curr);
                remaining -= 1;
                if i >= remaining {
                    i = 0;
                }
                fail_count = 0;
                continue;
            }
        }

        fail_count += 1;
        i += 1;
        if i >= remaining {
            i = 0;
        }
    }

    result
}

fn point_in_triangle(p: Point2, a: Point2, b: Point2, c: Point2) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

fn sign(p1: Point2, p2: Point2, p3: Point2) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

// ── Shader compilation helpers ────────────────────────────────────────

unsafe fn compile_shader(shader_type: u32, source: &str) -> u32 {
    let shader = gl::CreateShader(shader_type);
    let c_str = CString::new(source).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut success = 0i32;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut len = 0i32;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut i8);
        let msg = String::from_utf8_lossy(&buf);
        let kind = if shader_type == gl::VERTEX_SHADER { "VERTEX" } else { "FRAGMENT" };
        eprintln!("[Renderer] {} shader compile error: {}", kind, msg);
    }
    shader
}

unsafe fn create_program(vert_src: &str, frag_src: &str) -> u32 {
    let vs = compile_shader(gl::VERTEX_SHADER, vert_src);
    let fs = compile_shader(gl::FRAGMENT_SHADER, frag_src);
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);

    let mut success = 0i32;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success == 0 {
        let mut len = 0i32;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut i8);
        let msg = String::from_utf8_lossy(&buf);
        eprintln!("[Renderer] Program link error: {}", msg);
    }

    gl::DeleteShader(vs);
    gl::DeleteShader(fs);
    program
}

fn get_uniform(program: u32, name: &str) -> i32 {
    let c_name = CString::new(name).unwrap();
    unsafe { gl::GetUniformLocation(program, c_name.as_ptr()) }
}
