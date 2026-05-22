// AnimationPage - translated from C++ AnimationPage.h

use crate::color::Color;
use crate::rect::RectFrame;
use crate::types::RectGradient;
use crate::theme::{current_page_visuals, PageVisualTokens, ThemeColorTokens};
use crate::theme_tokens::{
    compose_page_header, compose_page_section, light_theme_colors, PageHeaderLayout,
};
use crate::ui::context::UIContext;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleKind {
    Fade,
    Scale,
    Move,
    Rotate,
}

#[derive(Debug, Clone)]
pub struct CardSpec {
    pub title: &'static str,
    pub line1: &'static str,
    pub line2: &'static str,
    pub badge: &'static str,
    pub kind: SampleKind,
}

#[derive(Debug, Clone, Copy)]
struct Layout {
    gap: f32,
    top_offset: f32,
    columns: i32,
    card_width: f32,
    card_height: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            gap: 18.0,
            top_offset: 98.0,
            columns: 1,
            card_width: 0.0,
            card_height: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Card data
// ---------------------------------------------------------------------------

fn cards() -> [CardSpec; 4] {
    [
        CardSpec {
            title: "Fade Alpha",
            line1: "Hover sample to fade between",
            line2: "two opacity states.",
            badge: ".hoverOpacity()",
            kind: SampleKind::Fade,
        },
        CardSpec {
            title: "Uniform Scale",
            line1: "Hover sample to scale",
            line2: "both axes together.",
            badge: ".hoverScale()",
            kind: SampleKind::Scale,
        },
        CardSpec {
            title: "Move XY",
            line1: "Hover sample to shift",
            line2: "x / y with one builder.",
            badge: ".hoverTranslate*",
            kind: SampleKind::Move,
        },
        CardSpec {
            title: "Triangle Rotate",
            line1: "Hover sample to rotate triangle",
            line2: "and blend plate color.",
            badge: "panel + polygon",
            kind: SampleKind::Rotate,
        },
    ]
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

fn make_layout(bounds: &RectFrame, content_y: f32) -> Layout {
    let mut layout = Layout::default();
    layout.top_offset = (content_y - bounds.y).max(0.0);
    layout.columns = if bounds.width >= 520.0 { 2 } else { 1 };
    layout.card_width = if layout.columns == 2 {
        (bounds.width - layout.gap) * 0.5
    } else {
        bounds.width
    };

    let available_height = (bounds.height - layout.top_offset - layout.gap).max(240.0);
    layout.card_height = if layout.columns == 2 {
        ((available_height - layout.gap) * 0.5).clamp(170.0, 210.0)
    } else {
        ((available_height - layout.gap * 3.0) * 0.25).clamp(116.0, 152.0)
    };
    layout
}

fn card_frame(bounds: &RectFrame, layout: &Layout, content_y: f32, index: i32) -> RectFrame {
    let column = index % layout.columns;
    let row = index / layout.columns;
    RectFrame::new(
        bounds.x + column as f32 * (layout.card_width + layout.gap),
        content_y + row as f32 * (layout.card_height + layout.gap),
        layout.card_width,
        layout.card_height,
    )
}

fn sample_frame(card: &RectFrame) -> RectFrame {
    let w = (84.0_f32).min(card.width * 0.24);
    let h = (56.0_f32).min(card.height * 0.30);
    let x = card.x + card.width - w - 26.0;

    let desired_y = card.y + card.height * 0.58 - h * 0.5;
    let min_y = card.y + card.height * 0.38;
    let max_y = card.y + card.height - h - (28.0_f32).max(card.height * 0.18);
    let y = if max_y >= min_y {
        desired_y.clamp(min_y, max_y)
    } else {
        desired_y
    };
    RectFrame::new(x, y, w, h)
}

// ---------------------------------------------------------------------------
// Sample gradient helper
// ---------------------------------------------------------------------------

fn sample_gradient(accent: &Color) -> RectGradient {
    RectGradient::corners(
        Color::new(
            (accent.r + 0.18).min(1.0),
            (accent.g + 0.14).min(1.0),
            (accent.b + 0.08).min(1.0),
            1.0,
        ),
        Color::new(
            (accent.r + 0.28).min(1.0),
            (accent.g + 0.08).min(1.0),
            (accent.b + 0.24).min(1.0),
            1.0,
        ),
        Color::new(
            (accent.r - 0.08).max(0.0),
            (accent.g - 0.14).max(0.0),
            (accent.b - 0.02).max(0.0),
            1.0,
        ),
        Color::new(
            (accent.r + 0.06).min(1.0),
            (accent.g - 0.10).max(0.0),
            (accent.b + 0.20).min(1.0),
            1.0,
        ),
    )
}

// ---------------------------------------------------------------------------
// AnimationPage
// ---------------------------------------------------------------------------

pub struct AnimationPage;

impl AnimationPage {
    pub fn compose(ui: &mut UIContext, id_prefix: &str, bounds: &RectFrame) {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let palette = light_theme_colors(); // TODO: use actual current theme
        let visuals = current_page_visuals(&palette);

        let header = compose_page_header(
            ui,
            id_prefix,
            bounds,
            "Animation Page",
            "Hover cards to preview DSL tracks. The samples use ui.panel / ui.polygon directly.",
            &visuals,
        );

        let layout = make_layout(bounds, header.content_y);
        let card_data = cards();
        for (index, card_spec) in card_data.iter().enumerate() {
            let frame = card_frame(bounds, &layout, header.content_y, index as i32);
            Self::compose_card(
                ui,
                &format!("{}.card.{}", id_prefix, index),
                &frame,
                card_spec,
                &palette,
                &visuals,
            );
        }
    }

    fn compose_badge(
        _ui: &mut UIContext,
        _id_prefix: &str,
        _visuals: &PageVisualTokens,
        _palette: &ThemeColorTokens,
        _x: f32,
        _y: f32,
        _text: &str,
    ) {
        // TODO: Renderer::measure_text_width for badge width
        // let badge_size = (14.0_f32).max(visuals.label_size - 1.0);
        // let scale = badge_size / 24.0;
        // let width = Renderer::measure_text_width(text, scale);
        // ui.panel(format!("{}.bg", id_prefix))
        //     .position(x, y).size(width + 20.0, badge_size + 12.0)
        //     .background(palette.surface_hover).rounding(10.0).build();
        // ui.label(format!("{}.label", id_prefix))
        //     .text(text).position(x + 10.0, y + badge_size + 3.0)
        //     .font_size(badge_size)
        //     .color(Color::new(palette.text.r, palette.text.g, palette.text.b, 0.82))
        //     .build();
    }

    fn compose_card(
        ui: &mut UIContext,
        id_prefix: &str,
        frame: &RectFrame,
        card: &CardSpec,
        palette: &ThemeColorTokens,
        visuals: &PageVisualTokens,
    ) {
        let accent = palette.primary;
        let dark = palette.dark;
        let sf = sample_frame(frame);
        let _sample_center_x = sf.x + sf.width * 0.5;
        let _sample_center_y = sf.y + sf.height * 0.5;
        let _card_title_size = visuals.header_subtitle_size;
        let _card_body_size = visuals.label_size;
        let _sample_gradient = sample_gradient(&accent);

        // Card background
        // TODO: ui.panel(format!("{}.card", id_prefix))
        //     .position(frame.x, frame.y)
        //     .size(frame.width, frame.height)
        //     .background(palette.surface)
        //     .rounding(16.0)
        //     .build();

        // Title
        // TODO: ui.label(format!("{}.title", id_prefix))
        //     .text(card.title)
        //     .position(frame.x + 24.0, frame.y + 40.0)
        //     .font_size(card_title_size)
        //     .color(Color::new(palette.text.r, palette.text.g, palette.text.b, 0.96))
        //     .build();

        // Line 1
        // TODO: ui.label(format!("{}.line1", id_prefix))
        //     .text(card.line1)
        //     .position(frame.x + 24.0, frame.y + 72.0)
        //     .font_size(card_body_size)
        //     .color(Color::new(palette.text.r, palette.text.g, palette.text.b, 0.66))
        //     .build();

        // Line 2
        // TODO: ui.label(format!("{}.line2", id_prefix))
        //     .text(card.line2)
        //     .position(frame.x + 24.0, frame.y + 90.0)
        //     .font_size(card_body_size)
        //     .color(Color::new(palette.text.r, palette.text.g, palette.text.b, 0.66))
        //     .build();

        // Badge
        Self::compose_badge(
            ui,
            &format!("{}.badge", id_prefix),
            visuals,
            palette,
            frame.x + 24.0,
            frame.y + frame.height - 38.0,
            card.badge,
        );

        // Sample shape per kind
        match card.kind {
            SampleKind::Fade => {
                let _rest_opacity = if dark { 0.36 } else { 0.48 };
                // TODO: ui.panel(format!("{}.sample.shape", id_prefix))
                //     .position(sf.x, sf.y)
                //     .size(sf.width, sf.height)
                //     .background(accent.r, accent.g, accent.b, 1.0)
                //     .gradient(sample_gradient)
                //     .rounding(12.0)
                //     .hover_opacity(rest_opacity, 1.0, 0.18)
                //     .build();
            }
            SampleKind::Scale => {
                // TODO: ui.panel(format!("{}.sample.shape", id_prefix))
                //     .position(sf.x, sf.y)
                //     .size(sf.width, sf.height)
                //     .background(accent.r, accent.g, accent.b, 1.0)
                //     .gradient(sample_gradient)
                //     .rounding(12.0)
                //     .hover_scale(1.0, 1.18, 0.18)
                //     .build();
            }
            SampleKind::Move => {
                // TODO: ui.panel(format!("{}.sample.shape", id_prefix))
                //     .position(sf.x, sf.y)
                //     .size(sf.width, sf.height)
                //     .background(accent.r, accent.g, accent.b, 1.0)
                //     .gradient(sample_gradient)
                //     .rounding(12.0)
                //     .hover_translate_x(0.0, 18.0, 0.18)
                //     .hover_translate_y(0.0, -8.0, 0.18)
                //     .build();
            }
            SampleKind::Rotate => {
                // Plate behind triangle
                // TODO: ui.panel(format!("{}.sample.plate", id_prefix))
                //     .position(sf.x - 10.0, sf.y - 6.0)
                //     .size(sf.width + 20.0, sf.height + 12.0)
                //     .background(accent.r, accent.g, accent.b, 0.08)
                //     .rounding(14.0)
                //     .hover_background(
                //         Color::new(accent.r, accent.g, accent.b, 0.08),
                //         Color::new(0.16, 0.82, 0.66, 0.18),
                //         0.18,
                //     )
                //     .build();

                // Triangle polygon
                // TODO: ui.polygon(format!("{}.sample.shape", id_prefix))
                //     .position(sample_center_x - 30.0, sample_center_y - 28.0)
                //     .size(60.0, 56.0)
                //     .background(accent.r, accent.g, accent.b, 0.94)
                //     .gradient(sample_gradient)
                //     .points(&[
                //         Point2 { x: 0.50, y: 0.00 },
                //         Point2 { x: 1.00, y: 1.00 },
                //         Point2 { x: 0.00, y: 1.00 },
                //     ])
                //     .hover_rotation(-16.0, 16.0, 0.18)
                //     .hover_opacity(0.40, 1.0, 0.18)
                //     .build();
            }
        }

        // Suppress unused variable warnings for variables referenced only in TODO blocks
        let _ = (dark, accent, sf);
    }
}
