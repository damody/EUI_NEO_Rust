// HomePage - translated from C++ HomePage.h

use crate::color::Color;
use crate::rect::RectFrame;
use crate::theme::{current_page_visuals, current_theme_colors, PageVisualTokens, ThemeColorTokens};
use crate::theme_tokens::{compose_page_header, compose_page_section, PageHeaderLayout};
use crate::ui::context::UIContext;

/// Callbacks for HomePage user interactions.
pub struct HomePageActions {
    pub on_randomize_theme_color: Option<Box<dyn Fn()>>,
    pub on_toggle_icon_accent: Option<Box<dyn Fn()>>,
    pub on_progress_change: Option<Box<dyn Fn(f32)>>,
    pub on_segmented_change: Option<Box<dyn Fn(i32)>>,
    pub on_input_change: Option<Box<dyn Fn(&str)>>,
    pub on_combo_change: Option<Box<dyn Fn(i32)>>,
}

impl Default for HomePageActions {
    fn default() -> Self {
        Self {
            on_randomize_theme_color: None,
            on_toggle_icon_accent: None,
            on_progress_change: None,
            on_segmented_change: None,
            on_input_change: None,
            on_combo_change: None,
        }
    }
}

pub struct HomePage;

impl HomePage {
    /// Static compose function -- mirrors C++ `HomePage::Compose`.
    #[allow(clippy::too_many_arguments)]
    pub fn compose(
        ui: &mut UIContext,
        id_prefix: &str,
        bounds: &RectFrame,
        icon_accent_enabled: bool,
        progress_value: f32,
        segmented_items: &[String],
        segmented_index: i32,
        input_text: &str,
        combo_items: &[String],
        combo_selection: i32,
        actions: &HomePageActions,
    ) {
        let palette = current_theme_colors_standalone();
        let visuals = current_page_visuals(&palette);

        let header = compose_page_header(
            ui,
            id_prefix,
            bounds,
            "Home Controls",
            "Basic widgets use the same page spacing and top-aligned layout.",
            &visuals,
        );

        let gap = visuals.section_gap;
        let actions_y = header.content_y;
        let wide_actions = bounds.width >= 420.0;
        let action_height = if wide_actions { 76.0 } else { 144.0 };
        let form_y = actions_y + action_height + gap;
        let form_height = (bounds.y + bounds.height - form_y).max(0.0);
        let button_gap = 12.0_f32;
        let stacked_button_gap = 6.0_f32;
        let button_top = if wide_actions { 18.0 } else { 12.0 };
        let button_width = if wide_actions {
            (bounds.width - button_gap * 2.0 - 40.0) / 3.0
        } else {
            (bounds.width - 40.0).max(0.0)
        };
        let button_x = bounds.x + 20.0;
        let form_inset = visuals.section_inset;
        let column_gap = visuals.section_gap;

        // Actions section background
        compose_page_section(
            ui,
            &format!("{}.actions", id_prefix),
            &RectFrame::new(bounds.x, actions_y, bounds.width, action_height),
            None,
            &visuals,
        );

        // Primary button
        let _primary_x = button_x;
        let _primary_y = actions_y + button_top;
        // TODO: ui.button(format!("{}.primary", id_prefix))
        //     .text("Primary")
        //     .position(button_x, actions_y + button_top)
        //     .size(button_width, 40.0)
        //     .style(ButtonStyle::Primary)
        //     .font_size(20.0)
        //     .on_click(|| { if let Some(ref f) = actions.on_randomize_theme_color { f(); } })
        //     .build();

        // Outline button
        let outline_x = if wide_actions {
            button_x + button_width + button_gap
        } else {
            button_x
        };
        let outline_y = if wide_actions {
            actions_y + button_top
        } else {
            actions_y + button_top + 40.0 + stacked_button_gap
        };
        let _ = (outline_x, outline_y);
        // TODO: ui.button(format!("{}.outline", id_prefix))
        //     .text("Outline")
        //     .position(outline_x, outline_y)
        //     .size(button_width, 40.0)
        //     .style(ButtonStyle::Outline)
        //     .font_size(20.0)
        //     .build();

        // Icon button
        let icon_x = if wide_actions {
            button_x + (button_width + button_gap) * 2.0
        } else {
            button_x
        };
        let icon_y = if wide_actions {
            actions_y + button_top
        } else {
            actions_y + button_top + (40.0 + stacked_button_gap) * 2.0
        };
        let _icon_accent = icon_accent_enabled;
        let _ = (icon_x, icon_y);
        // TODO: ui.button(format!("{}.icon", id_prefix))
        //     .text("Icon")
        //     .icon("\u{f013}")
        //     .icon_placement(ButtonIconPlacement::Trailing)
        //     .position(icon_x, icon_y)
        //     .size(button_width, 40.0)
        //     .font_size(20.0)
        //     .text_color(if icon_accent_enabled { palette.primary } else { palette.text })
        //     .on_click(|| { if let Some(ref f) = actions.on_toggle_icon_accent { f(); } })
        //     .build();

        if form_height <= 0.0 {
            return;
        }

        // Form section background
        compose_page_section(
            ui,
            &format!("{}.form", id_prefix),
            &RectFrame::new(bounds.x, form_y, bounds.width, form_height),
            None,
            &visuals,
        );

        let inner_x = bounds.x + form_inset;
        let inner_y = form_y + form_inset;
        let inner_width = (bounds.width - form_inset * 2.0).max(0.0);
        let inner_height = (form_height - form_inset * 2.0).max(0.0);
        if inner_width <= 0.0 || inner_height <= 0.0 {
            return;
        }

        let column_width = ((inner_width - column_gap) * 0.5).max(0.0);
        let left_x = inner_x;
        let left_y = inner_y;
        let right_x = inner_x + column_width + column_gap;
        let right_y = inner_y;
        let left_field_width = (column_width - 36.0).max(0.0);
        let right_field_width = (column_width - 36.0).max(0.0);
        let progress_value_x = (left_x + 18.0).max(left_x + column_width - 58.0);

        // -- Left column: Progress, Slider, Segmented --

        // TODO: ui.label(format!("{}.progress.label", id_prefix))
        //     .text("Progress")
        //     .position(left_x + 18.0, left_y + 28.0)
        //     .font_size(visuals.label_size)
        //     .build();

        let progress_pct = format!("{}%", (progress_value * 100.0) as i32);
        let _ = progress_pct;
        // TODO: ui.label(format!("{}.progress.value", id_prefix))
        //     .text(&progress_pct)
        //     .position(progress_value_x, left_y + 28.0)
        //     .font_size(16.0)
        //     .color(visuals.body_color)
        //     .build();

        // TODO: ui.progress(format!("{}.progress", id_prefix))
        //     .position(left_x + 18.0, left_y + 56.0)
        //     .size(left_field_width, 15.0)
        //     .value(progress_value)
        //     .build();

        // TODO: ui.slider(format!("{}.slider", id_prefix))
        //     .position(left_x + 18.0, left_y + 82.0)
        //     .size(left_field_width, 20.0)
        //     .value(progress_value)
        //     .on_change(|v| { if let Some(ref f) = actions.on_progress_change { f(v); } })
        //     .build();

        // TODO: ui.label(format!("{}.segmented.label", id_prefix))
        //     .text("Segment")
        //     .position(left_x + 18.0, left_y + 126.0)
        //     .font_size(visuals.label_size)
        //     .build();

        let _ = (segmented_items, segmented_index);
        // TODO: ui.segmented(format!("{}.segmented", id_prefix))
        //     .position(left_x + 18.0, left_y + 154.0)
        //     .size(left_field_width, 35.0)
        //     .items(segmented_items)
        //     .selected(segmented_index)
        //     .font_size(20.0)
        //     .on_change(|i| { if let Some(ref f) = actions.on_segmented_change { f(i); } })
        //     .build();

        // -- Right column: Input, Combo --

        // TODO: ui.label(format!("{}.input.label", id_prefix))
        //     .text("Input")
        //     .position(right_x + 18.0, right_y + 28.0)
        //     .font_size(visuals.label_size)
        //     .build();

        let _ = input_text;
        // TODO: ui.input(format!("{}.input", id_prefix))
        //     .position(right_x + 18.0, right_y + 56.0)
        //     .size(right_field_width, visuals.field_height)
        //     .placeholder("Type something...")
        //     .font_size(20.0)
        //     .text(input_text)
        //     .on_change(|t| { if let Some(ref f) = actions.on_input_change { f(t); } })
        //     .build();

        // TODO: ui.label(format!("{}.combo.label", id_prefix))
        //     .text("Combo")
        //     .position(right_x + 18.0, right_y + 108.0)
        //     .font_size(visuals.label_size)
        //     .build();

        let _ = (combo_items, combo_selection);
        // TODO: ui.combo(format!("{}.combo", id_prefix))
        //     .position(right_x + 18.0, right_y + 136.0)
        //     .size(right_field_width, visuals.field_height)
        //     .placeholder("Select an option")
        //     .font_size(20.0)
        //     .start_open(true)
        //     .items(combo_items)
        //     .selected(combo_selection)
        //     .on_change(|i| { if let Some(ref f) = actions.on_combo_change { f(i); } })
        //     .build();

        // Suppress unused warnings for layout variables used only by TODO builders
        let _ = (left_y, right_y, left_field_width, right_field_width, progress_value_x);
        let _ = button_width;
    }
}

/// Helper: build ThemeColorTokens from the default light/dark theme globals.
/// This mirrors C++ `CurrentThemeColors()` which reads the global `CurrentTheme` pointer.
/// In Rust we construct from the theme module helpers directly.
fn current_theme_colors_standalone() -> ThemeColorTokens {
    // Default to light theme; callers should wire this to the actual theme state.
    crate::theme_tokens::light_theme_colors()
}
