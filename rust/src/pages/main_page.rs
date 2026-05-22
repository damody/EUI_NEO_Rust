// MainPage - translated from C++ MainPage.h

use crate::color::Color;
use crate::rect::RectFrame;
use crate::renderer::renderer::Renderer;
use crate::state::UIState;
use crate::theme::{current_page_visuals, PageVisualTokens, ThemeColorTokens};
use crate::theme_tokens::{light_theme_colors, dark_theme_colors};
use crate::types::{RenderLayer, RectStyle};
use crate::ui::context::UIContext;

use crate::pages::home_page::{HomePage, HomePageActions};
use crate::pages::animation_page::AnimationPage;
use crate::pages::layout_page::{LayoutPage, LayoutPageActions};
use crate::pages::typography_page::TypographyPage;

// ---------------------------------------------------------------------------
// View enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainPageView {
    Home = 0,
    Animation = 1,
    Layout = 2,
    Typography = 3,
}

// ---------------------------------------------------------------------------
// Accent palette
// ---------------------------------------------------------------------------

const ACCENT_PALETTE: [Color; 10] = [
    Color { r: 0.20, g: 0.50, b: 0.90, a: 1.0 },
    Color { r: 0.12, g: 0.72, b: 0.78, a: 1.0 },
    Color { r: 0.15, g: 0.78, b: 0.48, a: 1.0 },
    Color { r: 0.88, g: 0.42, b: 0.18, a: 1.0 },
    Color { r: 0.92, g: 0.28, b: 0.46, a: 1.0 },
    Color { r: 0.56, g: 0.36, b: 0.96, a: 1.0 },
    Color { r: 0.96, g: 0.68, b: 0.18, a: 1.0 },
    Color { r: 0.78, g: 0.22, b: 0.78, a: 1.0 },
    Color { r: 0.32, g: 0.64, b: 0.24, a: 1.0 },
    Color { r: 0.88, g: 0.18, b: 0.24, a: 1.0 },
];

// ---------------------------------------------------------------------------
// Internal layout struct
// ---------------------------------------------------------------------------

struct MainLayout {
    sidebar_x: f32,
    sidebar_y: f32,
    sidebar_h: f32,
    content_x: f32,
    content_y: f32,
    content_w: f32,
    content_h: f32,
    current_content_offset_x: f32,
}

// ---------------------------------------------------------------------------
// MainPage
// ---------------------------------------------------------------------------

pub struct MainPage {
    ui: UIContext,
    current_view: MainPageView,
    page_reveal: f32,
    page_reveal_direction: i32,
    shell_padding: f32,
    sidebar_width: f32,
    content_inset: f32,
    progress_value: f32,
    home_icon_accent_enabled: bool,
    segmented_items: Vec<String>,
    segmented_index: i32,
    input_text: String,
    combo_items: Vec<String>,
    combo_selection: i32,
    layout_split: f32,
    random_seed: u32,
    accent_index: i32,
    state_version: u64,
    is_dark: bool,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            ui: UIContext::new(),
            current_view: MainPageView::Home,
            page_reveal: 1.0,
            page_reveal_direction: 1,
            shell_padding: 22.0,
            sidebar_width: 86.0,
            content_inset: 34.0,
            progress_value: 0.30,
            home_icon_accent_enabled: true,
            segmented_items: vec![
                "Apple".to_string(),
                "Banana".to_string(),
                "Cherry".to_string(),
            ],
            segmented_index: 0,
            input_text: String::new(),
            combo_items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
            ],
            combo_selection: -1,
            layout_split: 0.42,
            random_seed: 0xC0FF_EE11,
            accent_index: 0,
            state_version: 0,
            is_dark: true,
        }
    }

    // -- Public API (mirrors C++ Update / Draw / WantsContinuousUpdate) --

    pub fn update(&mut self, state: &UIState) {
        if self.page_reveal < 1.0 {
            let previous = self.page_reveal;
            self.page_reveal = lerp_f32(self.page_reveal, 1.0, state.delta_time * 11.0);
            if (1.0 - self.page_reveal).abs() < 0.01 {
                self.page_reveal = 1.0;
            }
            if (previous - self.page_reveal).abs() > 0.0001 {
                // TODO: ui.request_visual_refresh(0.18);
            }
        }

        let version_before = self.state_version;
        self.compose(state);
        self.ui.update(state);
        if self.state_version != version_before || self.ui.consume_recompose_request() {
            self.compose(state);
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        self.ui.draw(renderer);
    }

    /// Direct rendering: bypass UIContext and call renderer methods directly.
    /// Produces the same 55 draw commands as the C++ version for dump comparison.
    pub fn draw_direct(&self, renderer: &mut Renderer, state: &UIState) {
        let layout = self.make_layout(state);
        let palette = dark_theme_colors();
        let visuals = current_page_visuals(&palette);

        // Content area bounds (for page content inside glass panel)
        let bounds = RectFrame::new(
            layout.content_x + self.content_inset,
            layout.content_y + 18.0,
            layout.content_w - self.content_inset * 2.0,
            layout.content_h - 36.0,
        );

        // ─── 0-2: Backdrop circles ──────────────────────────────────────
        // [0] Red circle
        renderer.draw_rect_simple(
            layout.content_x + layout.content_w * 0.10 - 84.0,
            layout.content_y + 58.0,
            196.0, 196.0,
            &Color::new(0.98, 0.36, 0.36, 0.92),
            98.0,
        );
        // [1] Green circle
        renderer.draw_rect_simple(
            layout.content_x + layout.content_w * 0.58,
            layout.content_y + 86.0,
            164.0, 164.0,
            &Color::new(0.30, 0.92, 0.58, 0.88),
            82.0,
        );
        // [2] Blue circle
        renderer.draw_rect_simple(
            layout.content_x + layout.content_w * 0.30,
            layout.content_y + layout.content_h * 0.44,
            246.0, 246.0,
            &Color::new(0.34, 0.52, 1.00, 0.90),
            123.0,
        );

        // ─── 3: Glass panel ─────────────────────────────────────────────
        let glass_style = RectStyle {
            color: Color::new(0.15, 0.15, 0.18, 0.60),
            rounding: 16.0,
            blur_amount: self.progress_value * 0.15,
            shadow_blur: 20.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 10.0,
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.30),
            ..Default::default()
        };
        renderer.draw_rect(
            layout.content_x, layout.content_y,
            layout.content_w, layout.content_h,
            &glass_style,
        );

        // ─── 4-5: Page header ───────────────────────────────────────────
        let title_y = bounds.y + visuals.header_top_inset;
        let subtitle_y = title_y + visuals.header_title_gap;
        let content_y = subtitle_y + visuals.header_content_gap;

        // [4] Title: "Home Controls"
        renderer.draw_text(
            "Home Controls",
            bounds.x, title_y,
            &Color::new(1.0, 1.0, 1.0, 0.98),
            visuals.header_title_size / 24.0,
        );
        // [5] Subtitle
        renderer.draw_text(
            "Basic widgets use the same page spacing and top-aligned layout.",
            bounds.x, subtitle_y,
            &Color::new(1.0, 1.0, 1.0, 0.72),
            visuals.header_subtitle_size / 24.0,
        );

        // ─── Actions section ────────────────────────────────────────────
        let gap = visuals.section_gap;
        let actions_y = content_y;
        let wide_actions = bounds.width >= 420.0;
        let action_height = if wide_actions { 76.0 } else { 144.0 };
        let button_gap = 12.0_f32;
        let button_top = if wide_actions { 18.0 } else { 12.0 };
        let button_width = if wide_actions {
            (bounds.width - button_gap * 2.0 - 40.0) / 3.0
        } else {
            (bounds.width - 40.0).max(0.0)
        };
        let button_x = bounds.x + 20.0;

        // [6] Actions section background
        renderer.draw_rect_simple(
            bounds.x, actions_y,
            bounds.width, action_height,
            &palette.surface,
            visuals.section_rounding,
        );

        // [7] Primary button fill
        let primary_x = button_x;
        let primary_y = actions_y + button_top;
        renderer.draw_rect_simple(
            primary_x, primary_y,
            button_width, 40.0,
            &palette.primary,
            6.0,
        );
        // [8] "Primary" text
        let primary_text_w = renderer.measure_text_width("Primary", 20.0 / 24.0);
        renderer.draw_text(
            "Primary",
            primary_x + (button_width - primary_text_w) * 0.5,
            primary_y + 25.0,
            &Color::new(1.0, 1.0, 1.0, 1.0),
            20.0 / 24.0,
        );

        // [9-10] Outline button (border + inner)
        let outline_x = if wide_actions {
            button_x + button_width + button_gap
        } else {
            button_x
        };
        let outline_y = if wide_actions {
            actions_y + button_top
        } else {
            actions_y + button_top + 40.0 + 6.0
        };
        // [9] Border
        renderer.draw_rect_simple(
            outline_x, outline_y,
            button_width, 40.0,
            &palette.border,
            6.0,
        );
        // [10] Inner fill
        renderer.draw_rect_simple(
            outline_x + 1.0, outline_y + 1.0,
            button_width - 2.0, 38.0,
            &palette.background,
            5.0,
        );
        // [11] "Outline" text
        let outline_text_w = renderer.measure_text_width("Outline", 20.0 / 24.0);
        renderer.draw_text(
            "Outline",
            outline_x + (button_width - outline_text_w) * 0.5,
            outline_y + 25.0,
            &Color::new(1.0, 1.0, 1.0, 1.0),
            20.0 / 24.0,
        );

        // [12] Icon button fill
        let icon_x = if wide_actions {
            button_x + (button_width + button_gap) * 2.0
        } else {
            button_x
        };
        let icon_y = if wide_actions {
            actions_y + button_top
        } else {
            actions_y + button_top + (40.0 + 6.0) * 2.0
        };
        renderer.draw_rect_simple(
            icon_x, icon_y,
            button_width, 40.0,
            &palette.surface,
            6.0,
        );
        // [13] "Icon" text
        let icon_text_color = if self.home_icon_accent_enabled {
            palette.primary
        } else {
            palette.text
        };
        let icon_text_w = renderer.measure_text_width("Icon", 20.0 / 24.0);
        let icon_glyph_w = renderer.measure_text_width("\u{f013}", 20.0 / 24.0);
        let icon_total_w = icon_text_w + 8.0 + icon_glyph_w;
        let icon_text_start = icon_x + (button_width - icon_total_w) * 0.5;
        renderer.draw_text(
            "Icon",
            icon_text_start,
            icon_y + 25.0,
            &icon_text_color,
            20.0 / 24.0,
        );
        // [14] Icon glyph (trailing)
        renderer.draw_text(
            "\u{f013}",
            icon_text_start + icon_text_w + 8.0,
            icon_y + 27.5,
            &icon_text_color,
            20.0 / 24.0,
        );

        // ─── Form section ───────────────────────────────────────────────
        let form_y = actions_y + action_height + gap;
        let form_height = (bounds.y + bounds.height - form_y).max(0.0);
        if form_height <= 0.0 {
            self.draw_sidebar(renderer, &layout, &palette, &visuals);
            return;
        }

        let form_inset = visuals.section_inset;
        let column_gap = visuals.section_gap;

        // [15] Form section background
        renderer.draw_rect_simple(
            bounds.x, form_y,
            bounds.width, form_height,
            &palette.surface,
            visuals.section_rounding,
        );

        let inner_x = bounds.x + form_inset;
        let inner_y = form_y + form_inset;
        let inner_width = (bounds.width - form_inset * 2.0).max(0.0);
        let column_width = ((inner_width - column_gap) * 0.5).max(0.0);
        let left_x = inner_x;
        let left_y = inner_y;
        let right_x = inner_x + column_width + column_gap;
        let right_y = inner_y;
        let left_field_width = (column_width - 36.0).max(0.0);
        let right_field_width = (column_width - 36.0).max(0.0);
        let progress_value_x = (left_x + column_width - 58.0).max(left_x + 18.0);

        // ── Left column: Progress, Slider, Segmented ────────────────────

        // [16] "Progress" label
        renderer.draw_text(
            "Progress",
            left_x + 18.0, left_y + 28.0,
            &palette.text,
            visuals.label_size / 24.0,
        );
        // [17] "30%" value
        let progress_pct = format!("{}%", (self.progress_value * 100.0) as i32);
        renderer.draw_text(
            &progress_pct,
            progress_value_x, left_y + 28.0,
            &visuals.body_color,
            16.0 / 24.0,
        );

        // [18] Progress bar track
        renderer.draw_rect_simple(
            left_x + 18.0, left_y + 56.0,
            left_field_width, 15.0,
            &palette.surface_hover,
            7.5,
        );
        // [19] Progress bar fill
        renderer.draw_rect_simple(
            left_x + 18.0, left_y + 56.0,
            left_field_width * self.progress_value, 15.0,
            &palette.primary,
            7.5,
        );

        // [20] Slider track
        let slider_track_y = left_y + 82.0;
        let slider_height = 20.0;
        let track_h = 4.0;
        let track_y = slider_track_y + (slider_height - track_h) * 0.5;
        renderer.draw_rect_simple(
            left_x + 18.0, track_y,
            left_field_width, track_h,
            &Color::new(0.27, 0.27, 0.30, 1.0),
            2.0,
        );
        // [21] Slider fill
        renderer.draw_rect_simple(
            left_x + 18.0, track_y,
            left_field_width * self.progress_value, track_h,
            &Color::new(palette.primary.r, palette.primary.g, palette.primary.b, 0.86),
            2.0,
        );
        // [22] Slider thumb
        let thumb_size = 16.0;
        let thumb_x = left_x + 18.0 + left_field_width * self.progress_value - thumb_size * 0.5;
        let thumb_y = slider_track_y + (slider_height - thumb_size) * 0.5;
        renderer.draw_rect_simple(
            thumb_x, thumb_y,
            thumb_size, thumb_size,
            &Color::new(1.0, 1.0, 1.0, 1.0),
            8.0,
        );

        // [23] "Segment" label
        renderer.draw_text(
            "Segment",
            left_x + 18.0, left_y + 126.0,
            &palette.text,
            visuals.label_size / 24.0,
        );
        // [24] Segmented control container
        let seg_x = left_x + 18.0;
        let seg_y = left_y + 154.0;
        let seg_w = left_field_width;
        let seg_h = 35.0;
        renderer.draw_rect_simple(seg_x, seg_y, seg_w, seg_h, &palette.surface, 6.0);
        // [25] Selected segment indicator
        let seg_count = self.segmented_items.len() as f32;
        let seg_cell_w = seg_w / seg_count;
        let sel_x = seg_x + 2.0 + seg_cell_w * self.segmented_index as f32;
        let sel_w = seg_cell_w - 4.0;
        renderer.draw_rect_simple(sel_x, seg_y + 2.0, sel_w, seg_h - 4.0, &palette.primary, 5.0);
        // [26-28] Segment labels
        for (i, item) in self.segmented_items.iter().enumerate() {
            let item_center_x = seg_x + seg_cell_w * i as f32 + seg_cell_w * 0.5;
            let text_w = renderer.measure_text_width(item, 20.0 / 24.0);
            renderer.draw_text(
                item,
                item_center_x - text_w * 0.5,
                seg_y + 22.5,
                &Color::new(1.0, 1.0, 1.0, 1.0),
                20.0 / 24.0,
            );
        }

        // ── Right column: Input, Combo ──────────────────────────────────

        // [29] "Input" label
        renderer.draw_text(
            "Input",
            right_x + 18.0, right_y + 28.0,
            &palette.text,
            visuals.label_size / 24.0,
        );
        // [30] Input field background
        let input_x = right_x + 18.0;
        let input_y = right_y + 56.0;
        renderer.draw_rect_simple(
            input_x, input_y,
            right_field_width, visuals.field_height,
            &palette.surface,
            6.0,
        );
        // [31] Input border line
        renderer.draw_rect_simple(
            input_x, input_y,
            right_field_width, 1.0,
            &palette.border,
            0.0,
        );
        // [32] Placeholder "Type something..."
        renderer.draw_text(
            "Type something...",
            input_x + 10.0, input_y + visuals.field_height * 0.5 + 5.0,
            &Color::new(1.0, 1.0, 1.0, 0.50),
            20.0 / 24.0,
        );

        // [33] "Combo" label
        renderer.draw_text(
            "Combo",
            right_x + 18.0, right_y + 108.0,
            &palette.text,
            visuals.label_size / 24.0,
        );

        // Combo popup (drawn before combo field in C++ due to layer ordering)
        let combo_x = right_x + 18.0;
        let combo_y = right_y + 136.0;
        let combo_w = right_field_width;
        let combo_h = visuals.field_height;
        let popup_overlap = 1.0;
        let popup_item_h = 35.0;
        let popup_h = self.combo_items.len() as f32 * popup_item_h + popup_overlap;
        let popup_y = combo_y + combo_h - popup_overlap;

        // [34] Combo popup shadow/bg
        let popup_style = RectStyle {
            color: palette.surface,
            rounding: 10.0,
            shadow_blur: 18.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 8.0,
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.28),
            ..Default::default()
        };
        renderer.draw_rect(combo_x, popup_y, combo_w, popup_h, &popup_style);

        // [35-37] Popup items: "Item 1", "Item 2", "Item 3"
        for (i, item) in self.combo_items.iter().enumerate() {
            let item_y = popup_y + popup_overlap + popup_item_h * i as f32 + popup_item_h * 0.5 + 5.0;
            renderer.draw_text(
                item,
                combo_x + 10.0, item_y,
                &Color::new(1.0, 1.0, 1.0, 1.0),
                20.0 / 24.0,
            );
        }

        // [38] Combo field bg (active/focused state)
        renderer.draw_rect_simple(
            combo_x, combo_y,
            combo_w, combo_h,
            &palette.surface_active,
            6.0,
        );
        // [39] Combo border line
        renderer.draw_rect_simple(
            combo_x, combo_y,
            combo_w, 1.0,
            &palette.border,
            0.0,
        );
        // [40] Combo focus accent line
        renderer.draw_rect_simple(
            combo_x, combo_y,
            combo_w, 2.0,
            &palette.primary,
            0.0,
        );
        // [41] "Select an option" placeholder
        renderer.draw_text(
            "Select an option",
            combo_x + 10.0, combo_y + combo_h * 0.5 + 5.0,
            &Color::new(1.0, 1.0, 1.0, 0.50),
            20.0 / 24.0,
        );
        // [42] Chevron icon (U+F106 = angle-up, matching C++ open-state)
        renderer.draw_text(
            "\u{f106}",
            combo_x + combo_w - 25.0, combo_y + combo_h * 0.5 + 5.0,
            &Color::new(1.0, 1.0, 1.0, 1.0),
            20.0 / 24.0,
        );

        // ─── Sidebar ───────────────────────────────────────────────────
        self.draw_sidebar(renderer, &layout, &palette, &visuals);
    }

    /// Draw the sidebar with brand, nav items, and theme toggle.
    /// Uses C++ centeredOrigin logic for precise icon/text positioning.
    fn draw_sidebar(
        &self,
        renderer: &mut Renderer,
        layout: &MainLayout,
        palette: &ThemeColorTokens,
        _visuals: &PageVisualTokens,
    ) {
        let sb_x = layout.sidebar_x;
        let sb_y = layout.sidebar_y;
        let sb_w = self.sidebar_width;
        let sb_h = layout.sidebar_h;

        // C++ centeredOrigin: start + (extent - boundsSize) * 0.5 - boundsOffset
        let centered_origin = |start: f32, extent: f32, bounds_offset: f32, bounds_size: f32| -> f32 {
            start + (extent - bounds_size) * 0.5 - bounds_offset
        };

        // C++ centeredTextY: uses MeasureTextBounds to center vertically
        let centered_text_y = |renderer: &Renderer, frame_y: f32, frame_h: f32,
                                text: &str, scale: f32, fallback_h: f32| -> f32 {
            let bounds = renderer.measure_text_bounds(text, scale);
            let height = bounds.height.max(fallback_h);
            centered_origin(frame_y, frame_h, bounds.y, height)
        };

        // [43] Sidebar outer border
        renderer.draw_rect_simple(sb_x, sb_y, sb_w, sb_h, &palette.border, 20.0);

        // [44] Sidebar inner fill with shadow
        let inner_style = RectStyle {
            color: palette.surface,
            rounding: 19.0,
            shadow_blur: 18.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 8.0,
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.24),
            ..Default::default()
        };
        renderer.draw_rect(sb_x + 1.0, sb_y + 1.0, sb_w - 2.0, sb_h - 2.0, &inner_style);

        let show_labels = sb_w >= 128.0;

        // [45] "EUI" brand text (centered horizontally when narrow)
        let brand_primary_scale = 28.0 / 24.0;
        let brand_secondary_scale = 22.0 / 24.0;
        let primary_w = renderer.measure_text_width("EUI", brand_primary_scale);
        let secondary_w = renderer.measure_text_width("NEO", brand_secondary_scale);
        let primary_x = if show_labels { sb_x + 20.0 } else { sb_x + (sb_w - primary_w) * 0.5 };
        let secondary_x = if show_labels { sb_x + 20.0 } else { sb_x + (sb_w - secondary_w) * 0.5 };

        renderer.draw_text(
            "EUI", primary_x, sb_y + 44.0,
            &palette.primary,
            brand_primary_scale,
        );
        // [46] "NEO" brand text
        renderer.draw_text(
            "NEO", secondary_x, sb_y + 68.0,
            &Color::new(1.0, 1.0, 1.0, 0.82),
            brand_secondary_scale,
        );

        // Navigation items
        let nav_icons = ["\u{f015}", "\u{f04b}", "\u{f009}", "\u{f031}"];
        let nav_item_size = 52.0;
        let nav_start_y = sb_y + 112.0;
        let nav_gap = 10.0;
        let selected = self.current_view as usize;

        // [47] Selected nav item highlight
        let sel_nav_y = nav_start_y + (nav_item_size + nav_gap) * selected as f32;
        renderer.draw_rect_simple(
            sb_x + (sb_w - nav_item_size) * 0.5,
            sel_nav_y,
            nav_item_size, nav_item_size,
            &Color::new(palette.primary.r, palette.primary.g, palette.primary.b, 0.22),
            14.0,
        );

        // [48-51] Nav icons (using C++ centeredOrigin for precise positioning)
        let icon_scale = 20.0 / 24.0;
        for (i, icon) in nav_icons.iter().enumerate() {
            let frame_x = sb_x + (sb_w - nav_item_size) * 0.5;
            let frame_y = nav_start_y + (nav_item_size + nav_gap) * i as f32;
            let alpha = if i == selected { 0.98 } else { 0.60 };

            let icon_bounds = renderer.measure_text_bounds(icon, icon_scale);
            let icon_x = centered_origin(frame_x, nav_item_size, icon_bounds.x, icon_bounds.width);
            let icon_y = centered_text_y(renderer, frame_y, nav_item_size, icon, icon_scale, 20.0 * 0.8);

            renderer.draw_text(
                icon, icon_x, icon_y,
                &Color::new(1.0, 1.0, 1.0, alpha),
                icon_scale,
            );
        }

        // [52] Theme toggle button
        // C++: Lerp(surfaceHover, primary, toggleMix) with toggleMix = 0.14
        let toggle_y = sb_y + sb_h - nav_item_size - 16.0;
        let toggle_mix = 0.14_f32;
        let toggle_color = Color::new(
            palette.surface_hover.r + (palette.primary.r - palette.surface_hover.r) * toggle_mix,
            palette.surface_hover.g + (palette.primary.g - palette.surface_hover.g) * toggle_mix,
            palette.surface_hover.b + (palette.primary.b - palette.surface_hover.b) * toggle_mix,
            palette.surface_hover.a + (palette.primary.a - palette.surface_hover.a) * toggle_mix,
        );
        let toggle_frame_x = sb_x + (sb_w - nav_item_size) * 0.5;
        renderer.draw_rect_simple(
            toggle_frame_x, toggle_y,
            nav_item_size, nav_item_size,
            &toggle_color,
            14.0,
        );

        // [53-54] Theme toggle icons (moon/sun) using C++ centeredOrigin
        let toggle_icon_scale = 18.0 / 24.0;
        let moon_icon = "\u{f186}";
        let sun_icon = "\u{f185}";
        // Dark mode: moon visible (1 - themeBlend), sun invisible (themeBlend)
        // themeBlend = 0.0 for dark (moon visible), 1.0 for light (sun visible)
        let theme_blend = if self.is_dark { 0.0_f32 } else { 1.0_f32 };
        let icon_color_alpha = 0.96_f32;

        let moon_bounds = renderer.measure_text_bounds(moon_icon, toggle_icon_scale);
        let sun_bounds = renderer.measure_text_bounds(sun_icon, toggle_icon_scale);
        let icon_slot_x = toggle_frame_x;
        let icon_slot_width = nav_item_size;
        let moon_x = centered_origin(icon_slot_x, icon_slot_width, moon_bounds.x, moon_bounds.width);
        let sun_x = centered_origin(icon_slot_x, icon_slot_width, sun_bounds.x, sun_bounds.width);
        let moon_y = centered_text_y(renderer, toggle_y, nav_item_size, moon_icon, toggle_icon_scale, 18.0 * 0.8);
        let sun_y = centered_text_y(renderer, toggle_y, nav_item_size, sun_icon, toggle_icon_scale, 18.0 * 0.8);

        renderer.draw_text(
            moon_icon, moon_x, moon_y,
            &Color::new(1.0, 1.0, 1.0, icon_color_alpha * (1.0 - theme_blend)),
            toggle_icon_scale,
        );
        renderer.draw_text(
            sun_icon, sun_x, sun_y,
            &Color::new(1.0, 1.0, 1.0, icon_color_alpha * theme_blend),
            toggle_icon_scale,
        );
    }

    pub fn wants_continuous_update(&self) -> bool {
        self.ui.wants_continuous_update()
    }

    // -- Compose --

    fn compose(&mut self, state: &UIState) {
        let layout = self.make_layout(state);

        self.ui.begin("main");

        // Sidebar
        // TODO: ui.sidebar("sidebar")
        //     .position(layout.sidebar_x, layout.sidebar_y)
        //     .size(self.sidebar_width, layout.sidebar_h)
        //     .width(60.0, self.sidebar_width)
        //     .layer(RenderLayer::Chrome)
        //     .brand("EUI", "NEO")
        //     .selected_index(self.current_view as i32)
        //     .item("\u{f015}", "Home", || self.switch_view(MainPageView::Home))
        //     .item("\u{f04b}", "Animation", || self.switch_view(MainPageView::Animation))
        //     .item("\u{f009}", "Layout", || self.switch_view(MainPageView::Layout))
        //     .item("\u{f031}", "Typography", || self.switch_view(MainPageView::Typography))
        //     .theme_toggle(|| self.toggle_theme())
        //     .build();

        // Glass panel behind content
        // TODO: ui.glass_panel("content")
        //     .position(layout.content_x, layout.content_y)
        //     .size(layout.content_w, layout.content_h)
        //     .rounding(16.0)
        //     .blur(self.progress_value * 0.15)
        //     .layer(RenderLayer::Backdrop)
        //     .z_index(-2)
        //     .build();

        // Background decorative circles
        // TODO: ui.panel("bg.red")
        //     .position(layout.content_x + layout.content_w * 0.10 - 84.0, layout.content_y + 58.0)
        //     .size(196.0, 196.0)
        //     .background(0.98, 0.36, 0.36, 0.92)
        //     .layer(RenderLayer::Backdrop).rounding(98.0).z_index(-3).build();

        // TODO: ui.panel("bg.green")
        //     .position(layout.content_x + layout.content_w * 0.58, layout.content_y + 86.0)
        //     .size(164.0, 164.0)
        //     .background(0.30, 0.92, 0.58, 0.88)
        //     .layer(RenderLayer::Backdrop).rounding(82.0).z_index(-3).build();

        // TODO: ui.panel("bg.blue")
        //     .position(layout.content_x + layout.content_w * 0.30, layout.content_y + layout.content_h * 0.44)
        //     .size(246.0, 246.0)
        //     .background(0.34, 0.52, 1.0, 0.90)
        //     .layer(RenderLayer::Backdrop).rounding(123.0).z_index(-3).build();

        // Clip to content area and compose current page
        self.ui.push_clip(
            layout.content_x,
            layout.content_y,
            layout.content_w,
            layout.content_h,
        );
        let page_bounds = self.page_bounds(state);
        self.compose_current_page(&page_bounds);
        self.ui.pop_clip();

        self.ui.end();
    }

    // -- Theme toggle --

    fn toggle_theme(&mut self) {
        self.is_dark = !self.is_dark;
        self.state_version += 1;
        // TODO: ui.request_theme_refresh(0.18);
    }

    // -- View switching --

    fn switch_view(&mut self, view: MainPageView) {
        if view == self.current_view {
            return;
        }
        let previous_index = self.current_view as i32;
        let next_index = view as i32;
        self.current_view = view;
        self.page_reveal = 0.0;
        self.page_reveal_direction = if next_index >= previous_index { 1 } else { -1 };
        self.state_version += 1;
        // TODO: ui.request_visual_refresh(0.18);
    }

    // -- Layout computation --

    fn make_layout(&self, state: &UIState) -> MainLayout {
        let sidebar_x = self.shell_padding;
        let sidebar_y = self.shell_padding;
        let sidebar_h = (state.screen_h - self.shell_padding * 2.0).max(240.0);
        let content_x = sidebar_x + self.sidebar_width + 24.0;
        let content_y = self.shell_padding;
        let content_w = (state.screen_w - content_x - self.shell_padding).max(280.0);
        let content_h = (state.screen_h - self.shell_padding * 2.0).max(240.0);
        let current_content_offset_x =
            (1.0 - self.page_reveal) * 28.0 * self.page_reveal_direction as f32;

        MainLayout {
            sidebar_x,
            sidebar_y,
            sidebar_h,
            content_x,
            content_y,
            content_w,
            content_h,
            current_content_offset_x,
        }
    }

    fn page_bounds(&self, state: &UIState) -> RectFrame {
        let layout = self.make_layout(state);
        RectFrame::new(
            layout.content_x + self.content_inset + layout.current_content_offset_x,
            layout.content_y + 18.0,
            layout.content_w - self.content_inset * 2.0,
            layout.content_h - 36.0,
        )
    }

    fn compose_current_page(&mut self, bounds: &RectFrame) {
        match self.current_view {
            MainPageView::Home => {
                let actions = HomePageActions {
                    on_randomize_theme_color: None, // TODO: wire Self::randomize_theme_color
                    on_toggle_icon_accent: None,     // TODO: wire Self::toggle_home_icon_accent
                    on_progress_change: None,        // TODO: wire Self::set_progress_value
                    on_segmented_change: None,       // TODO: wire Self::set_segmented_index
                    on_input_change: None,           // TODO: wire Self::set_input_text
                    on_combo_change: None,           // TODO: wire Self::set_combo_selection
                };
                HomePage::compose(
                    &mut self.ui,
                    "home.page",
                    bounds,
                    self.home_icon_accent_enabled,
                    self.progress_value,
                    &self.segmented_items,
                    self.segmented_index,
                    &self.input_text,
                    &self.combo_items,
                    self.combo_selection,
                    &actions,
                );
            }
            MainPageView::Animation => {
                AnimationPage::compose(&mut self.ui, "animation.page", bounds);
            }
            MainPageView::Layout => {
                let actions = LayoutPageActions {
                    on_split_change: None, // TODO: wire Self::set_layout_split
                };
                LayoutPage::compose(
                    &mut self.ui,
                    "layout.page",
                    bounds,
                    self.layout_split,
                    &actions,
                );
            }
            MainPageView::Typography => {
                TypographyPage::compose(&mut self.ui, "typography.page", bounds);
            }
        }
    }

    // -- State mutators --

    fn set_progress_value(&mut self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        if (self.progress_value - clamped).abs() < 0.0001 {
            return;
        }
        self.progress_value = clamped;
        self.state_version += 1;
        // TODO: ui.request_visual_refresh(0.18);
    }

    fn set_segmented_index(&mut self, index: i32) {
        if self.segmented_index == index {
            return;
        }
        self.segmented_index = index;
        self.state_version += 1;
    }

    fn set_input_text(&mut self, text: &str) {
        if self.input_text == text {
            return;
        }
        self.input_text = text.to_string();
        self.state_version += 1;
    }

    fn set_combo_selection(&mut self, index: i32) {
        if self.combo_selection == index {
            return;
        }
        self.combo_selection = index;
        self.state_version += 1;
    }

    fn toggle_home_icon_accent(&mut self) {
        self.home_icon_accent_enabled = !self.home_icon_accent_enabled;
        self.state_version += 1;
    }

    fn randomize_theme_color(&mut self) {
        self.random_seed = self.random_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let mut next_index = (self.random_seed % ACCENT_PALETTE.len() as u32) as i32;
        if next_index == self.accent_index {
            next_index = (next_index + 1) % ACCENT_PALETTE.len() as i32;
        }
        self.accent_index = next_index;
        let _accent = ACCENT_PALETTE[self.accent_index as usize];
        // TODO: update LightTheme.primary and DarkTheme.primary with accent
        self.state_version += 1;
        // TODO: ui.request_theme_refresh(0.18);
    }

    fn set_layout_split(&mut self, value: f32) {
        let clamped = value.clamp(0.28, 0.72);
        if (self.layout_split - clamped).abs() < 0.0001 {
            return;
        }
        self.layout_split = clamped;
        self.state_version += 1;
    }
}

impl Default for MainPage {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
