use std::collections::VecDeque;
use crate::easing::{Easing, apply_easing};

/// Generic trait for types that support lerp interpolation.
pub trait Lerp: Clone + Default {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        self + (other - self) * t
    }
}

impl Lerp for crate::color::Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        crate::color::lerp_color(self, other, t)
    }
}

impl Lerp for crate::rect::RectFrame {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        crate::rect::lerp_rect(self, other, t)
    }
}

impl Lerp for crate::types::RectGradient {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        crate::types::lerp_gradient(self, other, t)
    }
}

impl Lerp for crate::types::RectTransform {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        crate::types::lerp_transform(self, other, t)
    }
}

impl Lerp for crate::types::RectStyle {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        crate::types::lerp_rect_style(self, other, t)
    }
}

impl Lerp for crate::types::PanelState {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        crate::types::PanelState {
            frame: self.frame.lerp(&other.frame, t),
            style: self.style.lerp(&other.style, t),
            border_width: self.border_width + (other.border_width - self.border_width) * t,
            border_color: self.border_color.lerp(&other.border_color, t),
        }
    }
}

struct Segment<T> {
    from: T,
    to: T,
    duration: f32,
    easing: Easing,
}

/// PropertyAnimation<T>: faithful translation of C++ PropertyAnimation template.
pub struct PropertyAnimation<T: Lerp> {
    current: T,
    active: Option<Segment<T>>,
    queued: VecDeque<Segment<T>>,
    elapsed: f32,
    running: bool,
    has_current: bool,
}

impl<T: Lerp> Default for PropertyAnimation<T> {
    fn default() -> Self {
        Self {
            current: T::default(),
            active: None,
            queued: VecDeque::new(),
            elapsed: 0.0,
            running: false,
            has_current: false,
        }
    }
}

impl<T: Lerp> PropertyAnimation<T> {
    pub fn new() -> Self { Self::default() }

    pub fn set_current(&mut self, value: T) {
        self.current = value;
        self.has_current = true;
    }

    pub fn play(&mut self, from: T, to: T, duration: f32, easing: Easing) {
        self.queued.clear();
        self.current = from.clone();
        self.active = Some(Segment { from, to, duration, easing });
        self.elapsed = 0.0;
        self.running = true;
        self.has_current = true;
    }

    pub fn play_to(&mut self, to: T, duration: f32, easing: Easing) {
        let from = self.resolve_current();
        self.play(from, to, duration, easing);
    }

    pub fn queue(&mut self, to: T, duration: f32, easing: Easing) {
        let from = self.resolve_queue_start();
        self.queued.push_back(Segment { from, to, duration, easing });
    }

    pub fn clear(&mut self) {
        self.queued.clear();
        self.elapsed = 0.0;
        self.running = false;
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if !self.running {
            if !self.queued.is_empty() {
                self.start_next_queued();
            } else {
                return false;
            }
        }

        if !self.running { return false; }

        self.elapsed += dt;
        if let Some(ref active) = self.active {
            if active.duration <= 0.0 {
                self.current = active.to.clone();
            } else {
                let t = (self.elapsed / active.duration).clamp(0.0, 1.0);
                self.current = active.from.lerp(&active.to, apply_easing(active.easing, t));
            }

            let done = active.duration <= 0.0 || self.elapsed >= active.duration;
            if done {
                self.current = active.to.clone();
                if !self.queued.is_empty() {
                    self.start_next_queued();
                } else {
                    self.running = false;
                }
            }
        }
        true
    }

    pub fn is_active(&self) -> bool {
        self.running || !self.queued.is_empty()
    }

    pub fn current(&self) -> &T {
        &self.current
    }

    fn resolve_current(&self) -> T {
        if self.has_current {
            self.current.clone()
        } else {
            T::default()
        }
    }

    fn resolve_queue_start(&self) -> T {
        if let Some(back) = self.queued.back() {
            return back.to.clone();
        }
        if let Some(ref active) = self.active {
            if self.running {
                return active.to.clone();
            }
        }
        self.resolve_current()
    }

    fn start_next_queued(&mut self) {
        if let Some(seg) = self.queued.pop_front() {
            self.current = seg.from.clone();
            self.has_current = true;
            self.elapsed = 0.0;
            self.running = true;
            self.active = Some(seg);
        } else {
            self.running = false;
        }
    }
}

pub type FloatAnimation = PropertyAnimation<f32>;
pub type ColorAnimation = PropertyAnimation<crate::color::Color>;
pub type GradientAnimation = PropertyAnimation<crate::types::RectGradient>;
pub type TransformAnimation = PropertyAnimation<crate::types::RectTransform>;
pub type RectStyleAnimation = PropertyAnimation<crate::types::RectStyle>;
pub type RectFrameAnimation = PropertyAnimation<crate::rect::RectFrame>;
pub type PanelStateAnimation = PropertyAnimation<crate::types::PanelState>;
