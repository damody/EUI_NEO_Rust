// LayoutPage - translated from C++ LayoutPage.h

use crate::color::Color;
use crate::rect::RectFrame;
use crate::theme::{current_page_visuals, PageVisualTokens, ThemeColorTokens};
use crate::theme_tokens::{
    compose_page_header, compose_page_section, light_theme_colors, PageHeaderLayout,
};
use crate::ui::context::{FlexDirection, UIContext};

/// Callbacks for LayoutPage user interactions.
pub struct LayoutPageActions {
    pub on_split_change: Option<Box<dyn Fn(f32)>>,
}

impl Default for LayoutPageActions {
    fn default() -> Self {
        Self {
            on_split_change: None,
        }
    }
}

pub struct LayoutPage;

impl LayoutPage {
    pub fn compose(
        ui: &mut UIContext,
        id_prefix: &str,
        bounds: &RectFrame,
        split_ratio: f32,
        _actions: &LayoutPageActions,
    ) {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let palette = light_theme_colors(); // TODO: use actual current theme
        let visuals = current_page_visuals(&palette);

        let header = compose_page_header(
            ui,
            id_prefix,
            bounds,
            "Layout",
            "row / column / flex.",
            &visuals,
        );

        let split = split_ratio.clamp(0.28, 0.72);
        let _slider_value = (split - 0.28) / 0.44;
        let gap = visuals.section_gap;
        let control_frame = RectFrame::new(bounds.x, header.content_y, bounds.width, 78.0);
        let demo_y = control_frame.y + control_frame.height + gap;
        let demo_frame = RectFrame::new(
            bounds.x,
            demo_y,
            bounds.width,
            (bounds.y + bounds.height - demo_y).max(0.0),
        );

        // Control section background
        compose_page_section(ui, &format!("{}.control", id_prefix), &control_frame, None, &visuals);

        // Control row: label + slider + value label
        // TODO: ui.row()
        //     .position(control_frame.x, control_frame.y)
        //     .size(control_frame.width, control_frame.height)
        //     .padding(20.0, 18.0)
        //     .gap(12.0)
        //     .content(|| {
        //         ui.label(format!("{}.control.label", id_prefix))
        //             .text("Split").font_size(visuals.label_size).build();
        //         ui.slider(format!("{}.control.slider", id_prefix))
        //             .flex(1.0).height(18.0).value(slider_value)
        //             .on_change(|v| {
        //                 if let Some(ref f) = actions.on_split_change {
        //                     f(0.28 + v.clamp(0.0, 1.0) * 0.44);
        //                 }
        //             }).build();
        //         ui.label(format!("{}.control.value", id_prefix))
        //             .text(&format!("{}%", (split * 100.0) as i32))
        //             .font_size(16.0).color(visuals.body_color).build();
        //     });

        if demo_frame.height <= 0.0 {
            return;
        }

        // Demo section background
        compose_page_section(ui, &format!("{}.demo", id_prefix), &demo_frame, None, &visuals);

        // Demo flex row: left column + divider + right column
        // TODO: ui.flex()
        //     .direction(FlexDirection::Row)
        //     .position(demo_frame.x, demo_frame.y)
        //     .size(demo_frame.width, demo_frame.height)
        //     .padding(20.0)
        //     .gap(gap)
        //     .content(|| {
        //         Self::compose_left_column(ui, &format!("{}.left", id_prefix), split, &visuals);
        //         ui.panel(format!("{}.divider", id_prefix))
        //             .width(gap).background(visuals.soft_accent_color).rounding(8.0).build();
        //         Self::compose_right_column(ui, &format!("{}.right", id_prefix), 1.0 - split, &visuals);
        //     });

        // Call column helpers even though their inner builders are TODO, to keep layout calc code live.
        Self::compose_left_column(ui, &format!("{}.left", id_prefix), split, &visuals);
        Self::compose_right_column(ui, &format!("{}.right", id_prefix), 1.0 - split, &visuals);
    }

    fn compose_left_column(
        _ui: &mut UIContext,
        _id_prefix: &str,
        _flex_weight: f32,
        _visuals: &PageVisualTokens,
    ) {
        // TODO: ui.column()
        //     .flex(flex_weight)
        //     .gap(12.0)
        //     .content(|| {
        //         ui.label(format!("{}.title", id_prefix))
        //             .text("column()").font_size(22.0)
        //             .margin(0.0, 16.0, 0.0, 0.0).build();
        //         ui.label(format!("{}.note", id_prefix))
        //             .text("Vertical stack. Auto fill width.")
        //             .font_size(18.0).color(visuals.body_color).build();
        //         ui.button(format!("{}.primary", id_prefix))
        //             .text("Full Width Button").style(ButtonStyle::Primary).build();
        //         ui.button(format!("{}.outline", id_prefix))
        //             .text("Second Item").style(ButtonStyle::Outline).build();
        //         ui.panel(format!("{}.fill", id_prefix))
        //             .flex(1.0).background(visuals.soft_accent_color).rounding(14.0).build();
        //     });
    }

    fn compose_right_column(
        _ui: &mut UIContext,
        _id_prefix: &str,
        _flex_weight: f32,
        _visuals: &PageVisualTokens,
    ) {
        // TODO: ui.column()
        //     .flex(flex_weight)
        //     .gap(12.0)
        //     .content(|| {
        //         ui.label(format!("{}.title", id_prefix))
        //             .text("flex()").font_size(22.0)
        //             .margin(0.0, 16.0, 0.0, 0.0).build();
        //         ui.label(format!("{}.note", id_prefix))
        //             .text("flex(n) splits remaining space.")
        //             .font_size(18.0).color(visuals.body_color).build();
        //         ui.row()
        //             .height(40.0).gap(12.0)
        //             .content(|| {
        //                 ui.button(format!("{}.one", id_prefix))
        //                     .flex(1.0).text("1x").build();
        //                 ui.button(format!("{}.two", id_prefix))
        //                     .flex(2.0).text("2x").build();
        //             });
        //         ui.progress(format!("{}.progress", id_prefix))
        //             .value(flex_weight.clamp(0.0, 1.0)).build();
        //         ui.panel(format!("{}.fill", id_prefix))
        //             .flex(1.0).background(visuals.muted_card_color).rounding(14.0).build();
        //     });
    }
}
