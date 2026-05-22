use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::Path;

use crate::color::Color;
use crate::rect::Point2;
use crate::types::{RectStyle, RectGradient};

/// Round to 2 decimal places.
fn r2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

/// Write an f32 with clean formatting (no trailing zeros for integers).
fn write_f32(out: &mut String, v: f32) {
    let r = r2(v);
    if r == r.floor() && r.abs() < 1e15 {
        write!(out, "{:.1}", r).unwrap();
    } else {
        write!(out, "{}", r).unwrap();
    }
}

/// Write an array of f32 values as a JSON array.
fn write_f32_array(out: &mut String, vals: &[f32]) {
    out.push('[');
    for (i, v) in vals.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write_f32(out, *v);
    }
    out.push(']');
}

/// Escape a string for JSON output.
fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A recorded draw command for JSON dump.
#[derive(Debug, Clone)]
pub struct DumpDrawCommand {
    pub cmd_type: String,
    pub rect: [f32; 4],
    pub clip_rect: [f32; 4],
    pub color: [f32; 4],
    pub radius: f32,
    pub thickness: f32,
    pub rotation: f32,
    pub blur_radius: f32,
    pub effect_alpha: f32,
    pub has_clip: bool,
    pub text: String,
    pub font_size: f32,
    pub has_gradient: bool,
    pub gradient_colors: [f32; 16],
    pub shadow_blur: f32,
    pub shadow_offset: [f32; 2],
    pub shadow_color: [f32; 4],
}

impl Default for DumpDrawCommand {
    fn default() -> Self {
        Self {
            cmd_type: String::new(),
            rect: [0.0; 4],
            clip_rect: [0.0; 4],
            color: [0.0; 4],
            radius: 0.0,
            thickness: 0.0,
            rotation: 0.0,
            blur_radius: 0.0,
            effect_alpha: 0.0,
            has_clip: false,
            text: String::new(),
            font_size: 0.0,
            has_gradient: false,
            gradient_colors: [0.0; 16],
            shadow_blur: 0.0,
            shadow_offset: [0.0; 2],
            shadow_color: [0.0; 4],
        }
    }
}

/// Global dump state for recording draw commands and writing JSON.
///
/// Usage:
///   1. Call `begin_recording()` to start capturing commands.
///   2. For each draw call, invoke the corresponding `record_*` method.
///   3. Call `end_recording_and_write(path)` to flush to a JSON file.
pub struct DumpState {
    recording: bool,
    commands: Vec<DumpDrawCommand>,
}

impl DumpState {
    pub fn new() -> Self {
        Self {
            recording: false,
            commands: Vec::new(),
        }
    }

    /// Start recording draw commands for a frame dump.
    pub fn begin_recording(&mut self) {
        self.recording = true;
        self.commands.clear();
    }

    /// Returns true if currently recording.
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Record a filled-rect draw command.
    pub fn record_rect(&mut self, x: f32, y: f32, w: f32, h: f32, style: &RectStyle) {
        if !self.recording {
            return;
        }
        let mut cmd = DumpDrawCommand::default();
        cmd.cmd_type = "FilledRect".to_string();
        cmd.rect = [x, y, w, h];
        cmd.color = [style.color.r, style.color.g, style.color.b, style.color.a];
        cmd.radius = style.rounding;
        cmd.blur_radius = style.blur_amount;
        cmd.rotation = style.transform.rotation_degrees;
        cmd.effect_alpha = style.color.a;
        cmd.has_gradient = style.gradient.enabled;
        if style.gradient.enabled {
            let corners = [
                &style.gradient.top_left,
                &style.gradient.top_right,
                &style.gradient.bottom_left,
                &style.gradient.bottom_right,
            ];
            for (i, c) in corners.iter().enumerate() {
                cmd.gradient_colors[i * 4] = c.r;
                cmd.gradient_colors[i * 4 + 1] = c.g;
                cmd.gradient_colors[i * 4 + 2] = c.b;
                cmd.gradient_colors[i * 4 + 3] = c.a;
            }
        }
        cmd.shadow_blur = style.shadow_blur;
        cmd.shadow_offset = [style.shadow_offset_x, style.shadow_offset_y];
        cmd.shadow_color = [
            style.shadow_color.r,
            style.shadow_color.g,
            style.shadow_color.b,
            style.shadow_color.a,
        ];
        self.commands.push(cmd);
    }

    /// Record a text draw command.
    pub fn record_text(&mut self, text: &str, x: f32, y: f32, color: &Color, scale: f32) {
        if !self.recording {
            return;
        }
        let mut cmd = DumpDrawCommand::default();
        cmd.cmd_type = "Text".to_string();
        cmd.rect = [x, y, 0.0, 0.0];
        cmd.color = [color.r, color.g, color.b, color.a];
        cmd.text = text.to_string();
        cmd.font_size = scale;
        cmd.effect_alpha = color.a;
        self.commands.push(cmd);
    }

    /// Record a polygon draw command.
    pub fn record_polygon(
        &mut self,
        points: &[Point2],
        fill: &Color,
        gradient: &RectGradient,
        stroke_width: f32,
        _stroke: &Color,
    ) {
        if !self.recording {
            return;
        }
        let mut cmd = DumpDrawCommand::default();
        cmd.cmd_type = "Polygon".to_string();
        if !points.is_empty() {
            let mut min_x = points[0].x;
            let mut min_y = points[0].y;
            let mut max_x = points[0].x;
            let mut max_y = points[0].y;
            for p in points {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
            }
            cmd.rect = [min_x, min_y, max_x - min_x, max_y - min_y];
        }
        cmd.color = [fill.r, fill.g, fill.b, fill.a];
        cmd.thickness = stroke_width;
        cmd.has_gradient = gradient.enabled;
        if gradient.enabled {
            let corners = [
                &gradient.top_left,
                &gradient.top_right,
                &gradient.bottom_left,
                &gradient.bottom_right,
            ];
            for (i, c) in corners.iter().enumerate() {
                cmd.gradient_colors[i * 4] = c.r;
                cmd.gradient_colors[i * 4 + 1] = c.g;
                cmd.gradient_colors[i * 4 + 2] = c.b;
                cmd.gradient_colors[i * 4 + 3] = c.a;
            }
        }
        cmd.effect_alpha = fill.a;
        self.commands.push(cmd);
    }

    /// Stop recording and write all captured commands to a JSON file.
    pub fn end_recording_and_write(&mut self, path: &str) {
        self.recording = false;

        let mut out = String::with_capacity(self.commands.len() * 512);
        writeln!(out, "{{").unwrap();
        writeln!(out, "  \"frame_command_count\": {},", self.commands.len()).unwrap();
        writeln!(out, "  \"commands\": [").unwrap();

        for (i, cmd) in self.commands.iter().enumerate() {
            out.push_str("    {\n");

            // index, type
            writeln!(out, "      \"index\": {},", i).unwrap();
            write!(out, "      \"type\": ").unwrap();
            write_json_string(&mut out, &cmd.cmd_type);
            out.push_str(",\n");

            // rect
            out.push_str("      \"rect\": ");
            write_f32_array(&mut out, &cmd.rect);
            out.push_str(",\n");

            // clip_rect
            out.push_str("      \"clip_rect\": ");
            write_f32_array(&mut out, &cmd.clip_rect);
            out.push_str(",\n");

            // color
            out.push_str("      \"color\": ");
            write_f32_array(&mut out, &cmd.color);
            out.push_str(",\n");

            // scalar fields
            out.push_str("      \"radius\": ");
            write_f32(&mut out, cmd.radius);
            out.push_str(",\n");

            out.push_str("      \"thickness\": ");
            write_f32(&mut out, cmd.thickness);
            out.push_str(",\n");

            out.push_str("      \"rotation\": ");
            write_f32(&mut out, cmd.rotation);
            out.push_str(",\n");

            out.push_str("      \"blur_radius\": ");
            write_f32(&mut out, cmd.blur_radius);
            out.push_str(",\n");

            out.push_str("      \"effect_alpha\": ");
            write_f32(&mut out, cmd.effect_alpha);
            out.push_str(",\n");

            // has_clip
            writeln!(out, "      \"has_clip\": {},", cmd.has_clip).unwrap();

            // text
            out.push_str("      \"text\": ");
            write_json_string(&mut out, &cmd.text);
            out.push_str(",\n");

            // font_size
            out.push_str("      \"font_size\": ");
            write_f32(&mut out, cmd.font_size);
            out.push_str(",\n");

            // gradient
            if cmd.has_gradient {
                out.push_str("      \"gradient\": {\n");
                out.push_str("        \"top_left\": ");
                write_f32_array(&mut out, &cmd.gradient_colors[0..4]);
                out.push_str(",\n");
                out.push_str("        \"top_right\": ");
                write_f32_array(&mut out, &cmd.gradient_colors[4..8]);
                out.push_str(",\n");
                out.push_str("        \"bottom_left\": ");
                write_f32_array(&mut out, &cmd.gradient_colors[8..12]);
                out.push_str(",\n");
                out.push_str("        \"bottom_right\": ");
                write_f32_array(&mut out, &cmd.gradient_colors[12..16]);
                out.push_str("\n");
                out.push_str("      },\n");
            } else {
                out.push_str("      \"gradient\": null,\n");
            }

            // shadow
            if cmd.shadow_blur > 0.0 || cmd.shadow_color[3] > 0.0 {
                out.push_str("      \"shadow\": {\n");
                out.push_str("        \"blur\": ");
                write_f32(&mut out, cmd.shadow_blur);
                out.push_str(",\n");
                out.push_str("        \"offset\": ");
                write_f32_array(&mut out, &cmd.shadow_offset);
                out.push_str(",\n");
                out.push_str("        \"color\": ");
                write_f32_array(&mut out, &cmd.shadow_color);
                out.push_str("\n");
                out.push_str("      },\n");
            } else {
                out.push_str("      \"shadow\": null,\n");
            }

            // transform (placeholder)
            out.push_str("      \"transform\": null\n");

            // close object
            out.push_str("    }");
            if i + 1 < self.commands.len() {
                out.push(',');
            }
            out.push('\n');
        }

        writeln!(out, "  ]").unwrap();
        writeln!(out, "}}").unwrap();

        let file_path = Path::new(path);
        match std::fs::File::create(file_path) {
            Ok(mut f) => {
                if let Err(e) = f.write_all(out.as_bytes()) {
                    eprintln!("[EUI-NEO Rust] Failed to write dump file: {}", e);
                } else {
                    eprintln!(
                        "[EUI-NEO Rust] Dumped {} draw commands to {}",
                        self.commands.len(),
                        file_path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "[EUI-NEO Rust] Failed to create dump file {}: {}",
                    file_path.display(),
                    e
                );
            }
        }

        self.commands.clear();
    }
}
