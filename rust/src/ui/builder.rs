use crate::color::Color;
use crate::types::{RectGradient, RenderLayer, Anchor, UIShadow};
use crate::ui::primitive::{UIPrimitive, UIClipRect};

/// Layout build information collected during builder chain.
#[derive(Debug, Clone, Default)]
pub struct LayoutBuildInfo {
    pub has_x: bool,
    pub has_y: bool,
    pub has_width: bool,
    pub has_height: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub flex: f32,
    pub margin_left: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
}

/// Common builder methods shared by all node builders.
///
/// Since Rust doesn't have CRTP, we use a trait with methods that mutate
/// a primitive and layout info. Concrete builders implement the two accessor
/// methods and inherit all the chainable setters.
pub trait UIBuilderOps {
    /// Access the primitive being configured.
    fn primitive_mut(&mut self) -> &mut UIPrimitive;

    /// Access the layout build info being configured.
    fn layout_build_mut(&mut self) -> &mut LayoutBuildInfo;

    fn x(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().x = value;
        self.layout_build_mut().has_x = true;
        self.layout_build_mut().x = value;
        self
    }

    fn y(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().y = value;
        self.layout_build_mut().has_y = true;
        self.layout_build_mut().y = value;
        self
    }

    fn position(&mut self, x: f32, y: f32) -> &mut Self {
        self.primitive_mut().x = x;
        self.primitive_mut().y = y;
        let lb = self.layout_build_mut();
        lb.has_x = true;
        lb.has_y = true;
        lb.x = x;
        lb.y = y;
        self
    }

    fn width(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().width = value;
        self.layout_build_mut().has_width = true;
        self.layout_build_mut().width = value;
        self
    }

    fn height(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().height = value;
        self.layout_build_mut().has_height = true;
        self.layout_build_mut().height = value;
        self
    }

    fn size(&mut self, w: f32, h: f32) -> &mut Self {
        self.primitive_mut().width = w;
        self.primitive_mut().height = h;
        let lb = self.layout_build_mut();
        lb.has_width = true;
        lb.has_height = true;
        lb.width = w;
        lb.height = h;
        self
    }

    fn min_width(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().min_width = value;
        self
    }

    fn min_height(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().min_height = value;
        self
    }

    fn max_width(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().max_width = value;
        self
    }

    fn max_height(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().max_height = value;
        self
    }

    fn anchor(&mut self, value: Anchor) -> &mut Self {
        self.primitive_mut().anchor = value;
        self
    }

    fn scale(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().scale_x = value;
        self.primitive_mut().scale_y = value;
        self
    }

    fn scale_x(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().scale_x = value;
        self
    }

    fn scale_y(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().scale_y = value;
        self
    }

    fn rotation(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().rotation = value;
        self
    }

    fn translate_x(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().translate_x = value;
        self
    }

    fn translate_y(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().translate_y = value;
        self
    }

    fn opacity(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().opacity = value.clamp(0.0, 1.0);
        self
    }

    fn flex(&mut self, value: f32) -> &mut Self {
        self.layout_build_mut().flex = value.max(0.0);
        self
    }

    fn margin_uniform(&mut self, value: f32) -> &mut Self {
        let v = value.max(0.0);
        let lb = self.layout_build_mut();
        lb.margin_left = v;
        lb.margin_top = v;
        lb.margin_right = v;
        lb.margin_bottom = v;
        self
    }

    fn margin_hv(&mut self, h: f32, v: f32) -> &mut Self {
        let lb = self.layout_build_mut();
        lb.margin_left = h.max(0.0);
        lb.margin_right = h.max(0.0);
        lb.margin_top = v.max(0.0);
        lb.margin_bottom = v.max(0.0);
        self
    }

    fn margin(&mut self, left: f32, top: f32, right: f32, bottom: f32) -> &mut Self {
        let lb = self.layout_build_mut();
        lb.margin_left = left.max(0.0);
        lb.margin_top = top.max(0.0);
        lb.margin_right = right.max(0.0);
        lb.margin_bottom = bottom.max(0.0);
        self
    }

    fn clip_rect(&mut self, x: f32, y: f32, w: f32, h: f32) -> &mut Self {
        let p = self.primitive_mut();
        p.has_clip_rect = w > 0.0 && h > 0.0;
        p.clip_rect = UIClipRect {
            x,
            y,
            width: w,
            height: h,
        };
        self
    }

    fn clear_clip_rect(&mut self) -> &mut Self {
        let p = self.primitive_mut();
        p.has_clip_rect = false;
        p.clip_rect = UIClipRect::default();
        self
    }

    fn clip_to_parent(&mut self, value: bool) -> &mut Self {
        self.primitive_mut().clip_to_parent = value;
        self
    }

    fn escape_clip(&mut self) -> &mut Self {
        let p = self.primitive_mut();
        p.clip_to_parent = false;
        p.has_clip_rect = false;
        p.clip_rect = UIClipRect::default();
        self
    }

    fn background_color(&mut self, color: Color) -> &mut Self {
        self.primitive_mut().background = color;
        self
    }

    fn background(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.primitive_mut().background = Color::new(r, g, b, a);
        self
    }

    fn gradient(&mut self, value: RectGradient) -> &mut Self {
        self.primitive_mut().gradient = value;
        self
    }

    fn rounding(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().rounding = value;
        self
    }

    fn border(&mut self, width: f32, color: Color) -> &mut Self {
        self.primitive_mut().border_width = width;
        self.primitive_mut().border_color = color;
        self
    }

    fn blur(&mut self, value: f32) -> &mut Self {
        self.primitive_mut().blur = value;
        self
    }

    fn shadow(&mut self, blur: f32, offset_y: f32, color: Color) -> &mut Self {
        let p = self.primitive_mut();
        p.shadow.blur = blur;
        p.shadow.offset_y = offset_y;
        p.shadow.color = color;
        self
    }

    fn shadow_full(&mut self, blur: f32, offset_x: f32, offset_y: f32, color: Color) -> &mut Self {
        let p = self.primitive_mut();
        p.shadow.blur = blur;
        p.shadow.offset_x = offset_x;
        p.shadow.offset_y = offset_y;
        p.shadow.color = color;
        self
    }

    fn visible(&mut self, value: bool) -> &mut Self {
        self.primitive_mut().visible = value;
        self
    }

    fn enabled(&mut self, value: bool) -> &mut Self {
        self.primitive_mut().enabled = value;
        self
    }

    fn layer(&mut self, value: RenderLayer) -> &mut Self {
        self.primitive_mut().render_layer = value;
        self
    }

    fn popup_layer(&mut self) -> &mut Self {
        self.primitive_mut().render_layer = RenderLayer::Popup;
        self.escape_clip()
    }

    fn z_index(&mut self, value: i32) -> &mut Self {
        self.primitive_mut().z_index = value;
        self
    }
}
