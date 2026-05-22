use std::collections::HashMap;

use crate::rect::RectFrame;

/// A loaded glyph with its own GL texture (per-character, matching C++ approach).
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub texture_id: u32,
    /// Render size (full texture including SDF padding).
    pub render_width: f32,
    pub render_height: f32,
    /// Render bearing (offset from origin to top-left of render texture).
    pub render_bearing_x: f32,
    pub render_bearing_y: f32,
    /// Visible size (tight bounding box of the glyph, no padding).
    pub visible_width: f32,
    pub visible_height: f32,
    /// Visible bearing (offset from origin to visible bounding box).
    pub visible_bearing_x: f32,
    pub visible_bearing_y: f32,
    /// Horizontal advance in native pixel units.
    pub advance: f32,
    /// The pixel size at which this glyph was rasterized.
    pub base_pixel_size: f32,
    pub is_sdf: bool,
}

/// Font manager using per-character textures.
///
/// Supports multiple font sources (matching C++ FontSources vector).
/// Each glyph gets its own GL_RED texture.
pub struct FontAtlas {
    characters: HashMap<u32, GlyphInfo>,
}

impl FontAtlas {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
        }
    }

    /// Load a font file and rasterize glyphs in [start_char..=end_char] as individual textures.
    ///
    /// `font_size` is the rasterization pixel size (e.g., 72.0 for UI font, 96.0 for icons).
    /// Glyphs are stored with their `base_pixel_size` so the renderer can scale correctly.
    pub fn load_font(
        &mut self,
        path: &str,
        font_size: f32,
        start_char: u32,
        end_char: u32,
        _use_sdf: bool,
    ) -> bool {
        let font_data = match std::fs::read(path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("[FontAtlas] Failed to read font file '{}': {}", path, e);
                return false;
            }
        };

        let settings = fontdue::FontSettings {
            scale: font_size,
            ..fontdue::FontSettings::default()
        };

        let font = match fontdue::Font::from_bytes(font_data, settings) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[FontAtlas] Failed to parse font file '{}': {}", path, e);
                return false;
            }
        };

        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        let mut loaded_count = 0u32;

        for codepoint in start_char..end_char {
            let ch = match char::from_u32(codepoint) {
                Some(c) => c,
                None => continue,
            };

            // Skip if already loaded
            if self.characters.contains_key(&codepoint) {
                continue;
            }

            // Check if glyph exists in this font
            if font.lookup_glyph_index(ch) == 0 && codepoint != 0 {
                continue;
            }

            let (metrics, bitmap) = font.rasterize(ch, font_size);

            let advance = metrics.advance_width;
            let glyph_w = metrics.width as u32;
            let glyph_h = metrics.height as u32;

            // Visible bearing: convert fontdue to stbtt convention.
            // fontdue ymin: bottom of glyph in Y-up from baseline (positive = above baseline)
            // stbtt visibleY0: top of glyph in Y-down from baseline (negative = above baseline)
            // top_y_up = ymin + height; stbtt_y0 = -top_y_up = -(ymin + height)
            let visible_bearing_x = metrics.xmin as f32;
            let visible_bearing_y = -(metrics.ymin as f32 + metrics.height as f32);
            let visible_width = metrics.width as f32;
            let visible_height = metrics.height as f32;

            if glyph_w == 0 || glyph_h == 0 || bitmap.is_empty() {
                self.characters.insert(codepoint, GlyphInfo {
                    texture_id: 0,
                    render_width: 0.0,
                    render_height: 0.0,
                    render_bearing_x: 0.0,
                    render_bearing_y: 0.0,
                    visible_width: 0.0,
                    visible_height: 0.0,
                    visible_bearing_x: 0.0,
                    visible_bearing_y: 0.0,
                    advance,
                    base_pixel_size: font_size,
                    is_sdf: false,
                });
                loaded_count += 1;
                continue;
            }

            // Create GL texture
            let mut texture: u32 = 0;
            unsafe {
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    glyph_w as i32,
                    glyph_h as i32,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    bitmap.as_ptr() as *const _,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }

            // render bearing: same as visible for non-SDF
            let render_bearing_x = metrics.xmin as f32;
            let render_bearing_y = -(metrics.ymin as f32 + metrics.height as f32);

            self.characters.insert(codepoint, GlyphInfo {
                texture_id: texture,
                render_width: glyph_w as f32,
                render_height: glyph_h as f32,
                render_bearing_x,
                render_bearing_y,
                visible_width,
                visible_height,
                visible_bearing_x,
                visible_bearing_y,
                advance,
                base_pixel_size: font_size,
                is_sdf: false,
            });
            loaded_count += 1;
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        eprintln!(
            "[FontAtlas] Loaded {} glyphs from '{}' at {}px (total: {})",
            loaded_count, path, font_size, self.characters.len()
        );
        true
    }

    /// Look up a glyph by Unicode codepoint.
    pub fn get_glyph(&self, codepoint: u32) -> Option<&GlyphInfo> {
        self.characters.get(&codepoint)
    }

    /// Resolve the glyph scale for a given normalized scale.
    /// Matches C++: `glyphScale = 24.0 * normalizedScale / basePixelSize`
    fn resolve_glyph_scale(base_pixel_size: f32, normalized_scale: f32) -> f32 {
        let base = base_pixel_size.max(1.0);
        24.0 * normalized_scale / base
    }

    /// Resolve the requested text pixel size for space width calculation.
    /// Matches C++: `ResolveRequestedTextPixelSize(scale) = 24.0 * scale`
    fn resolve_requested_pixel_size(normalized_scale: f32) -> f32 {
        24.0 * normalized_scale
    }

    /// Measure the width of a string at the given normalized scale.
    /// Matches C++ `Renderer::MeasureTextWidth`.
    pub fn measure_text_width(&self, text: &str, scale: f32) -> f32 {
        let mut width = 0.0f32;
        for ch in text.chars() {
            let codepoint = ch as u32;
            if ch == ' ' {
                width += Self::resolve_requested_pixel_size(scale) * 0.3;
            } else if let Some(glyph) = self.characters.get(&codepoint) {
                let glyph_scale = Self::resolve_glyph_scale(glyph.base_pixel_size, scale);
                width += glyph.advance * glyph_scale;
            } else {
                width += Self::resolve_requested_pixel_size(scale) * 0.3;
            }
        }
        width
    }

    /// Measure the bounding box of rendered text.
    /// Matches C++ `Renderer::MeasureTextBounds`.
    /// Returns RectFrame where x/y are the visible bearing offsets.
    pub fn measure_text_bounds(&self, text: &str, scale: f32) -> RectFrame {
        if text.is_empty() {
            return RectFrame::ZERO;
        }

        let mut pen_x = 0.0f32;
        let mut has_glyph_bounds = false;
        let mut min_x = 0.0f32;
        let mut min_y = 0.0f32;
        let mut max_x = 0.0f32;
        let mut max_y = 0.0f32;

        for ch in text.chars() {
            let codepoint = ch as u32;
            if ch == ' ' {
                pen_x += Self::resolve_requested_pixel_size(scale) * 0.3;
                continue;
            }

            if let Some(glyph) = self.characters.get(&codepoint) {
                let glyph_scale = Self::resolve_glyph_scale(glyph.base_pixel_size, scale);
                let glyph_x = pen_x + glyph.visible_bearing_x * glyph_scale;
                let glyph_y = glyph.visible_bearing_y * glyph_scale;
                let glyph_w = glyph.visible_width * glyph_scale;
                let glyph_h = glyph.visible_height * glyph_scale;

                if glyph_w > 0.0 && glyph_h > 0.0 {
                    if !has_glyph_bounds {
                        min_x = glyph_x;
                        min_y = glyph_y;
                        max_x = glyph_x + glyph_w;
                        max_y = glyph_y + glyph_h;
                        has_glyph_bounds = true;
                    } else {
                        min_x = min_x.min(glyph_x);
                        min_y = min_y.min(glyph_y);
                        max_x = max_x.max(glyph_x + glyph_w);
                        max_y = max_y.max(glyph_y + glyph_h);
                    }
                }

                pen_x += glyph.advance * glyph_scale;
            }
        }

        if !has_glyph_bounds {
            return RectFrame::new(0.0, 0.0, pen_x, 0.0);
        }

        RectFrame::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Release all GL textures.
    pub fn destroy(&mut self) {
        for (_, glyph) in &self.characters {
            if glyph.texture_id != 0 {
                unsafe {
                    gl::DeleteTextures(1, &glyph.texture_id);
                }
            }
        }
        self.characters.clear();
    }
}
