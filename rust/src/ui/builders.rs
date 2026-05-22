// builders.rs — Builder structs for all 10 control types.
// Uses Take-Modify-Return pattern to avoid borrow conflicts with UIContext.

use crate::color::Color;
use crate::ui::builder::{LayoutBuildInfo, UIBuilderOps};
use crate::ui::context::UIContext;
use crate::ui::node::UINode;
use crate::ui::primitive::UIPrimitive;

use crate::components::label::LabelNode;
use crate::components::button::{ButtonNode, ButtonStyle, ButtonIconPlacement};
use crate::components::panel::{PanelNode, GlassPanelNode};
use crate::components::progress_bar::ProgressBarNode;
use crate::components::slider::SliderNode;
use crate::components::segmented_control::SegmentedControlNode;
use crate::components::input_box::InputBoxNode;
use crate::components::combo_box::ComboBoxNode;
use crate::components::sidebar::{SidebarNode, ItemSpec};

// ===========================================================================
// Macro: shared builder boilerplate
// ===========================================================================

macro_rules! impl_builder_ops {
    ($builder:ty) => {
        impl<'a> UIBuilderOps for $builder {
            fn primitive_mut(&mut self) -> &mut UIPrimitive {
                self.node.primitive_mut()
            }
            fn layout_build_mut(&mut self) -> &mut LayoutBuildInfo {
                &mut self.layout_build
            }
        }
    };
}

macro_rules! define_builder {
    ($name:ident) => {
        pub struct $name<'a> {
            pub(crate) ctx: &'a mut UIContext,
            pub(crate) node: Box<dyn UINode>,
            pub(crate) key: String,
            pub(crate) layout_build: LayoutBuildInfo,
        }

        impl_builder_ops!($name<'a>);

        impl<'a> $name<'a> {
            pub fn build(self) {
                self.ctx.return_and_finalize(self.key, self.node, self.layout_build);
            }
        }
    };
}

// ===========================================================================
// LabelBuilder
// ===========================================================================

define_builder!(LabelBuilder);

impl<'a> LabelBuilder<'a> {
    pub fn text(mut self, value: &str) -> Self {
        if let Some(label) = self.node.as_any_mut().downcast_mut::<LabelNode>() {
            label.text = value.to_string();
        }
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        if let Some(label) = self.node.as_any_mut().downcast_mut::<LabelNode>() {
            label.font_size = value;
        }
        self
    }

    pub fn color(mut self, value: Color) -> Self {
        if let Some(label) = self.node.as_any_mut().downcast_mut::<LabelNode>() {
            label.color = value;
            label.use_theme_color = false;
        }
        self
    }

    pub fn use_theme_color(mut self, value: bool) -> Self {
        if let Some(label) = self.node.as_any_mut().downcast_mut::<LabelNode>() {
            label.use_theme_color = value;
        }
        self
    }
}

// ===========================================================================
// ButtonBuilder
// ===========================================================================

define_builder!(ButtonBuilder);

impl<'a> ButtonBuilder<'a> {
    pub fn text(mut self, value: &str) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.text = value.to_string();
        }
        self
    }

    pub fn icon(mut self, value: &str) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.icon = value.to_string();
        }
        self
    }

    pub fn icon_placement(mut self, value: ButtonIconPlacement) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.icon_placement = value;
        }
        self
    }

    pub fn style(mut self, value: ButtonStyle) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.style = value;
        }
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.font_size = value;
        }
        self
    }

    pub fn text_color(mut self, value: Color) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.text_color_override = Some(value);
        }
        self
    }

    pub fn hover_scale(mut self, idle: f32, hover: f32, duration: f32) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.hover_scale_idle = idle;
            btn.hover_scale_hover = hover;
            btn.hover_scale_duration = duration;
        }
        self
    }

    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        if let Some(btn) = self.node.as_any_mut().downcast_mut::<ButtonNode>() {
            btn.on_click = Some(Box::new(callback));
        }
        self
    }
}

// ===========================================================================
// PanelBuilder
// ===========================================================================

define_builder!(PanelBuilder);

// PanelBuilder only has UIBuilderOps methods — no component-specific setters.

// ===========================================================================
// GlassPanelBuilder
// ===========================================================================

define_builder!(GlassPanelBuilder);

// GlassPanelBuilder only has UIBuilderOps methods.

// ===========================================================================
// ProgressBarBuilder
// ===========================================================================

define_builder!(ProgressBarBuilder);

impl<'a> ProgressBarBuilder<'a> {
    pub fn value(mut self, v: f32) -> Self {
        if let Some(pb) = self.node.as_any_mut().downcast_mut::<ProgressBarNode>() {
            pb.value = v.clamp(0.0, 1.0);
        }
        self
    }
}

// ===========================================================================
// SliderBuilder
// ===========================================================================

define_builder!(SliderBuilder);

impl<'a> SliderBuilder<'a> {
    pub fn value(mut self, v: f32) -> Self {
        if let Some(s) = self.node.as_any_mut().downcast_mut::<SliderNode>() {
            s.value = v.clamp(0.0, 1.0);
        }
        self
    }

    pub fn on_change(mut self, callback: impl Fn(f32) + 'static) -> Self {
        if let Some(s) = self.node.as_any_mut().downcast_mut::<SliderNode>() {
            s.on_change = Some(Box::new(callback));
        }
        self
    }
}

// ===========================================================================
// SegmentedControlBuilder
// ===========================================================================

define_builder!(SegmentedControlBuilder);

impl<'a> SegmentedControlBuilder<'a> {
    pub fn items(mut self, items: &[String]) -> Self {
        if let Some(sc) = self.node.as_any_mut().downcast_mut::<SegmentedControlNode>() {
            sc.items = items.to_vec();
        }
        self
    }

    pub fn selected(mut self, index: i32) -> Self {
        if let Some(sc) = self.node.as_any_mut().downcast_mut::<SegmentedControlNode>() {
            sc.selected_index = index;
        }
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        if let Some(sc) = self.node.as_any_mut().downcast_mut::<SegmentedControlNode>() {
            sc.font_size = value;
        }
        self
    }

    pub fn on_change(mut self, callback: impl Fn(i32, &str) + 'static) -> Self {
        if let Some(sc) = self.node.as_any_mut().downcast_mut::<SegmentedControlNode>() {
            sc.on_change = Some(Box::new(callback));
        }
        self
    }
}

// ===========================================================================
// InputBoxBuilder
// ===========================================================================

define_builder!(InputBoxBuilder);

impl<'a> InputBoxBuilder<'a> {
    pub fn placeholder(mut self, value: &str) -> Self {
        if let Some(ib) = self.node.as_any_mut().downcast_mut::<InputBoxNode>() {
            ib.placeholder = value.to_string();
        }
        self
    }

    pub fn text(mut self, value: &str) -> Self {
        if let Some(ib) = self.node.as_any_mut().downcast_mut::<InputBoxNode>() {
            ib.text = value.to_string();
        }
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        if let Some(ib) = self.node.as_any_mut().downcast_mut::<InputBoxNode>() {
            ib.font_size = value;
        }
        self
    }

    pub fn on_change(mut self, callback: impl Fn(&str) + 'static) -> Self {
        if let Some(ib) = self.node.as_any_mut().downcast_mut::<InputBoxNode>() {
            ib.on_change = Some(Box::new(callback));
        }
        self
    }

    pub fn on_enter(mut self, callback: impl Fn(&str) + 'static) -> Self {
        if let Some(ib) = self.node.as_any_mut().downcast_mut::<InputBoxNode>() {
            ib.on_enter = Some(Box::new(callback));
        }
        self
    }
}

// ===========================================================================
// ComboBoxBuilder
// ===========================================================================

define_builder!(ComboBoxBuilder);

impl<'a> ComboBoxBuilder<'a> {
    pub fn items(mut self, items: &[String]) -> Self {
        if let Some(cb) = self.node.as_any_mut().downcast_mut::<ComboBoxNode>() {
            cb.items = items.to_vec();
        }
        self
    }

    pub fn placeholder(mut self, value: &str) -> Self {
        if let Some(cb) = self.node.as_any_mut().downcast_mut::<ComboBoxNode>() {
            cb.placeholder = value.to_string();
        }
        self
    }

    pub fn selected(mut self, index: i32) -> Self {
        if let Some(cb) = self.node.as_any_mut().downcast_mut::<ComboBoxNode>() {
            cb.selected_index = index;
        }
        self
    }

    pub fn start_open(mut self, value: bool) -> Self {
        if let Some(cb) = self.node.as_any_mut().downcast_mut::<ComboBoxNode>() {
            if !cb.open_state_initialized {
                cb.is_open = value;
                cb.open_state_initialized = true;
            }
        }
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        if let Some(cb) = self.node.as_any_mut().downcast_mut::<ComboBoxNode>() {
            cb.font_size = value;
        }
        self
    }

    pub fn on_change(mut self, callback: impl Fn(i32, &str) + 'static) -> Self {
        if let Some(cb) = self.node.as_any_mut().downcast_mut::<ComboBoxNode>() {
            cb.on_change = Some(Box::new(callback));
        }
        self
    }
}

// ===========================================================================
// SidebarBuilder
// ===========================================================================

define_builder!(SidebarBuilder);

impl<'a> SidebarBuilder<'a> {
    pub fn brand(mut self, primary: &str, secondary: &str) -> Self {
        if let Some(sb) = self.node.as_any_mut().downcast_mut::<SidebarNode>() {
            sb.brand_primary = primary.to_string();
            sb.brand_secondary = secondary.to_string();
        }
        self
    }

    pub fn item(mut self, icon: &str, label: &str, on_click: impl Fn() + 'static) -> Self {
        if let Some(sb) = self.node.as_any_mut().downcast_mut::<SidebarNode>() {
            sb.items.push(ItemSpec {
                icon: icon.to_string(),
                label: label.to_string(),
                on_click: Some(Box::new(on_click)),
            });
        }
        self
    }

    pub fn selected_index(mut self, index: i32) -> Self {
        if let Some(sb) = self.node.as_any_mut().downcast_mut::<SidebarNode>() {
            sb.selected_index = index;
        }
        self
    }

    pub fn on_theme_toggle(mut self, callback: impl Fn() + 'static) -> Self {
        if let Some(sb) = self.node.as_any_mut().downcast_mut::<SidebarNode>() {
            sb.on_theme_toggle = Some(Box::new(callback));
        }
        self
    }
}
