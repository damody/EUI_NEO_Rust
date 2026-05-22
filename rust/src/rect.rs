use crate::color::Color;

/// Axis-aligned rectangle (position + size).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RectFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectFrame {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width
            && py >= self.y && py <= self.y + self.height
    }

    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn bottom(&self) -> f32 { self.y + self.height }
}

pub fn lerp_rect(a: &RectFrame, b: &RectFrame, t: f32) -> RectFrame {
    let t = t.clamp(0.0, 1.0);
    RectFrame {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
    }
}

/// Bounding box (x, y, w, h) used for measure results.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RectBounds {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}
