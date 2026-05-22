// TypographyPage - translated from C++ TypographyPage.h

use crate::color::Color;
use crate::rect::RectFrame;
use crate::theme::{current_page_visuals, PageVisualTokens, ThemeColorTokens};
use crate::theme_tokens::{
    compose_page_header, compose_page_section, light_theme_colors, PageHeaderLayout,
};
use crate::ui::context::UIContext;

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

struct SectionText {
    title: &'static str,
    subtitle: &'static str,
}

#[derive(Debug, Clone)]
pub struct PreviewRow {
    pub size: f32,
    pub sample: &'static str,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const META_FONT_SIZE: f32 = 18.0;
const HEADER_TITLE_SIZE: f32 = 22.0;
const HEADER_SUBTITLE_SIZE: f32 = 16.0;
const WIDE_SAMPLE_OFFSET: f32 = 108.0;
const ROW_GAP: f32 = 8.0;
const ROW_BOTTOM_PADDING: f32 = 14.0;
const HEADER_BOTTOM_PADDING: f32 = 18.0;

const ICON_SAMPLE_TEXT: &str =
    "\u{f031}   \u{f015}   \u{f013}   \u{f107}   \u{f185}   \u{f186}";
const MIXED_HERO_TEXT: &str = "SDF keeps large counters clean";
const MIXED_BODY_TEXT: &str = "0123456789  ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const MIXED_BODY2_TEXT: &str = "The quick brown fox jumps over the lazy dog.";
const MIXED_BODY3_TEXT: &str =
    "\u{4E2D}\u{6587}\u{56DE}\u{9000}\u{9884}\u{89C8}  \u{4F60}\u{597D}\u{FF0C}\u{4E16}\u{754C}  \u{5B57}\u{4F53}\u{6DF7}\u{6392}";

// ---------------------------------------------------------------------------
// Section text constructors
// ---------------------------------------------------------------------------

fn scale_section() -> SectionText {
    SectionText {
        title: "Type Scale",
        subtitle: "Preview the same renderer at small and large text sizes.",
    }
}

fn mixed_section() -> SectionText {
    SectionText {
        title: "Mixed Content",
        subtitle: "Latin, numerals, punctuation, and CJK fallback in one place.",
    }
}

fn icon_section() -> SectionText {
    SectionText {
        title: "Icon Sizes",
        subtitle: "Font Awesome glyphs previewed across multiple display sizes.",
    }
}

// ---------------------------------------------------------------------------
// Row data
// ---------------------------------------------------------------------------

fn type_rows() -> [PreviewRow; 7] {
    [
        PreviewRow { size: 12.0, sample: "Caption 12  SDF sample 012345  ABC" },
        PreviewRow { size: 14.0, sample: "Body 14  Aa Bb Cc 012345  Sharp edges" },
        PreviewRow { size: 18.0, sample: "Reading size 18  The quick brown fox \u{4F60}\u{597D}" },
        PreviewRow { size: 24.0, sample: "Title 24  Vector-like curves and counters" },
        PreviewRow { size: 32.0, sample: "Display 32  Mixed Latin / \u{4E2D}\u{6587} glyphs" },
        PreviewRow { size: 44.0, sample: "Poster 44 typography" },
        PreviewRow { size: 60.0, sample: "Hero 60 \u{8DDD}\u{79BB}\u{573A} SDF" },
    ]
}

fn icon_rows() -> [PreviewRow; 5] {
    [
        PreviewRow { size: 16.0, sample: ICON_SAMPLE_TEXT },
        PreviewRow { size: 20.0, sample: ICON_SAMPLE_TEXT },
        PreviewRow { size: 24.0, sample: ICON_SAMPLE_TEXT },
        PreviewRow { size: 32.0, sample: ICON_SAMPLE_TEXT },
        PreviewRow { size: 48.0, sample: ICON_SAMPLE_TEXT },
    ]
}

// ---------------------------------------------------------------------------
// Measurement helpers
// ---------------------------------------------------------------------------

fn size_label(size: f32) -> String {
    format!("{} px", size as i32)
}

/// Estimate text height from font size.
/// In the C++ version this calls `Renderer::MeasureTextBounds`. Here we use a
/// heuristic until the renderer is wired up.
fn text_height(_text: &str, font_size: f32) -> f32 {
    // TODO: Renderer::measure_text_bounds(text, font_size / 24.0).height.max(font_size * 0.80)
    font_size * 1.2
}

fn text_bottom(_text: &str, font_size: f32, anchor_y: f32) -> f32 {
    // TODO: let bounds = Renderer::measure_text_bounds(text, font_size / 24.0);
    // anchor_y + bounds.y + bounds.height.max(font_size * 0.80)
    anchor_y + font_size * 1.2
}

fn section_header_height(section: &SectionText, visuals: &PageVisualTokens) -> f32 {
    let title_bottom = text_bottom(section.title, HEADER_TITLE_SIZE, visuals.section_inset);
    let subtitle_bottom =
        text_bottom(section.subtitle, HEADER_SUBTITLE_SIZE, visuals.section_inset + 28.0);
    title_bottom.max(subtitle_bottom) + HEADER_BOTTOM_PADDING
}

fn row_height(row: &PreviewRow, compact: bool) -> f32 {
    let label_h = text_height(&size_label(row.size), META_FONT_SIZE);
    let sample_h = text_height(row.sample, row.size);
    if compact {
        label_h + ROW_GAP + sample_h + ROW_BOTTOM_PADDING
    } else {
        label_h.max(sample_h) + ROW_BOTTOM_PADDING
    }
}

fn measure_rows_section_height(
    section: &SectionText,
    rows: &[PreviewRow],
    compact: bool,
    visuals: &PageVisualTokens,
) -> f32 {
    let mut height = section_header_height(section, visuals);
    for (i, row) in rows.iter().enumerate() {
        height += row_height(row, compact);
        if i + 1 < rows.len() {
            height += ROW_GAP;
        }
    }
    height
}

fn measure_mixed_section_height(visuals: &PageVisualTokens) -> f32 {
    let y = section_header_height(&mixed_section(), visuals);
    let candidates = [
        text_bottom(MIXED_HERO_TEXT, 30.0, y),
        text_bottom(MIXED_BODY_TEXT, 18.0, y + 42.0),
        text_bottom(MIXED_BODY2_TEXT, 18.0, y + 72.0),
        text_bottom(MIXED_BODY3_TEXT, 24.0, y + 102.0),
    ];
    let max_val = candidates.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    max_val + ROW_BOTTOM_PADDING
}

// ---------------------------------------------------------------------------
// TypographyPage
// ---------------------------------------------------------------------------

pub struct TypographyPage;

impl TypographyPage {
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
            "Typography",
            "SDF text preview across different sizes, mixed content, and icon rendering.",
            &visuals,
        );

        let viewport = RectFrame::new(
            bounds.x,
            header.content_y,
            bounds.width,
            (bounds.y + bounds.height - header.content_y).max(0.0),
        );
        if viewport.height <= 0.0 {
            return;
        }

        compose_page_section(
            ui,
            &format!("{}.viewport", id_prefix),
            &viewport,
            None,
            &visuals,
        );

        let compact = viewport.width < 640.0;
        let content_width = (viewport.width - visuals.section_inset * 2.0).max(0.0);
        let tr = type_rows();
        let ir = icon_rows();
        let scale_height =
            measure_rows_section_height(&scale_section(), &tr, compact, &visuals);
        let mixed_height = measure_mixed_section_height(&visuals);
        let icon_height =
            measure_rows_section_height(&icon_section(), &ir, compact, &visuals);
        let _content_height = visuals.section_inset
            + scale_height
            + visuals.section_gap
            + mixed_height
            + visuals.section_gap
            + icon_height;

        // TODO: ui.scroll_area(format!("{}.scroll", id_prefix),
        //     viewport.x, viewport.y, viewport.width, viewport.height, content_height,
        //     |_scroll_offset| { ... });

        // Compose sections at their computed Y positions (would be inside scroll callback).
        let mut y = viewport.y + visuals.section_inset;

        Self::compose_rows_section(
            ui,
            &format!("{}.scale", id_prefix),
            &RectFrame::new(viewport.x + visuals.section_inset, y, content_width, scale_height),
            &scale_section(),
            &tr,
            compact,
            &visuals,
        );
        y += scale_height + visuals.section_gap;

        Self::compose_mixed_section(
            ui,
            &format!("{}.mixed", id_prefix),
            &RectFrame::new(viewport.x + visuals.section_inset, y, content_width, mixed_height),
            &visuals,
        );
        y += mixed_height + visuals.section_gap;

        Self::compose_rows_section(
            ui,
            &format!("{}.icons", id_prefix),
            &RectFrame::new(viewport.x + visuals.section_inset, y, content_width, icon_height),
            &icon_section(),
            &ir,
            compact,
            &visuals,
        );
    }

    fn compose_section_header(
        _ui: &mut UIContext,
        _id_prefix: &str,
        _frame: &RectFrame,
        _section: &SectionText,
        _visuals: &PageVisualTokens,
    ) {
        // TODO: ui.label(format!("{}.title", id_prefix))
        //     .text(section.title)
        //     .position(frame.x + visuals.section_inset, frame.y + visuals.section_inset)
        //     .font_size(HEADER_TITLE_SIZE).color(visuals.title_color).build();
        // TODO: ui.label(format!("{}.subtitle", id_prefix))
        //     .text(section.subtitle)
        //     .position(frame.x + visuals.section_inset, frame.y + visuals.section_inset + 28.0)
        //     .font_size(HEADER_SUBTITLE_SIZE).color(visuals.body_color).build();
    }

    fn compose_preview_row(
        _ui: &mut UIContext,
        _id_prefix: &str,
        _row: &PreviewRow,
        _left_x: f32,
        _sample_x: f32,
        _row_top: f32,
        _compact: bool,
        _label_color: &Color,
    ) {
        // TODO: Renderer::measure_text_bounds for label and sample
        // let label = size_label(row.size);
        // let label_y = row_top - label_bounds.y;
        // let sample_top = if compact { row_top + text_height(&label, META_FONT_SIZE) + ROW_GAP } else { row_top };
        // let sample_y = sample_top - sample_bounds.y;
        // ui.label(format!("{}.label", id_prefix)).text(&label).position(left_x, label_y)
        //     .font_size(META_FONT_SIZE).color(*label_color).build();
        // ui.label(format!("{}.sample", id_prefix)).text(row.sample).position(sample_x, sample_y)
        //     .font_size(row.size).build();
    }

    fn compose_rows_section(
        ui: &mut UIContext,
        id_prefix: &str,
        frame: &RectFrame,
        section: &SectionText,
        rows: &[PreviewRow],
        compact: bool,
        visuals: &PageVisualTokens,
    ) {
        compose_page_section(
            ui,
            &format!("{}.panel", id_prefix),
            frame,
            None,
            visuals,
        );
        Self::compose_section_header(ui, id_prefix, frame, section, visuals);

        let left_x = frame.x + visuals.section_inset;
        let sample_x = if compact { left_x } else { left_x + WIDE_SAMPLE_OFFSET };
        let mut row_top = frame.y + section_header_height(section, visuals);
        for (i, row) in rows.iter().enumerate() {
            Self::compose_preview_row(
                ui,
                &format!("{}.row.{}", id_prefix, i),
                row,
                left_x,
                sample_x,
                row_top,
                compact,
                &visuals.body_color,
            );
            row_top += row_height(row, compact);
            if i + 1 < rows.len() {
                row_top += ROW_GAP;
            }
        }
    }

    fn compose_mixed_section(
        ui: &mut UIContext,
        id_prefix: &str,
        frame: &RectFrame,
        visuals: &PageVisualTokens,
    ) {
        let section = mixed_section();
        let _x = frame.x + visuals.section_inset;
        let _y = frame.y + section_header_height(&section, visuals);

        compose_page_section(
            ui,
            &format!("{}.panel", id_prefix),
            frame,
            None,
            visuals,
        );
        Self::compose_section_header(ui, id_prefix, frame, &section, visuals);

        // TODO: ui.label(format!("{}.hero", id_prefix))
        //     .text(MIXED_HERO_TEXT).position(x, y).font_size(30.0).build();
        // TODO: ui.label(format!("{}.body", id_prefix))
        //     .text(MIXED_BODY_TEXT).position(x, y + 42.0).font_size(18.0).color(visuals.body_color).build();
        // TODO: ui.label(format!("{}.body2", id_prefix))
        //     .text(MIXED_BODY2_TEXT).position(x, y + 72.0).font_size(18.0).color(visuals.body_color).build();
        // TODO: ui.label(format!("{}.body3", id_prefix))
        //     .text(MIXED_BODY3_TEXT).position(x, y + 102.0).font_size(24.0).build();
    }
}
