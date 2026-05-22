use std::collections::HashMap;
use crate::rect::RectFrame;
use crate::ui::primitive::{UIPrimitive, UIClipRect};
use crate::ui::node::UINode;
use crate::ui::builder::LayoutBuildInfo;
use crate::ui::builders::*;

use crate::components::label::LabelNode;
use crate::components::button::ButtonNode;
use crate::components::panel::{PanelNode, GlassPanelNode};
use crate::components::progress_bar::ProgressBarNode;
use crate::components::slider::SliderNode;
use crate::components::segmented_control::SegmentedControlNode;
use crate::components::input_box::InputBoxNode;
use crate::components::combo_box::ComboBoxNode;
use crate::components::sidebar::SidebarNode;

/// Flex layout direction (row = horizontal, column = vertical).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

// ---------------------------------------------------------------------------
// Internal layout types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Offset {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct LayoutItem {
    node_key: String,
    container_index: Option<usize>,
    build: LayoutBuildInfo,
}

#[derive(Debug, Clone)]
struct LayoutState {
    direction: FlexDirection,
    build: LayoutBuildInfo,
    gap: f32,
    padding_left: f32,
    padding_top: f32,
    padding_right: f32,
    padding_bottom: f32,
    children: Vec<LayoutItem>,
}

impl LayoutState {
    fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            build: LayoutBuildInfo::default(),
            gap: 0.0,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// UIContext: retained-mode compose/layout engine
// ---------------------------------------------------------------------------

/// UIContext manages the retained node tree, compose lifecycle, layout, and draw ordering.
pub struct UIContext {
    page_id: String,
    compose_stamp: u64,
    nodes: HashMap<String, Box<dyn UINode>>,
    order: Vec<String>,
    draw_order: Vec<String>,
    clip_stack: Vec<UIClipRect>,
    offset_stack: Vec<Offset>,
    owned_layouts: Vec<LayoutState>,
    layout_stack: Vec<usize>,
    base_context_offset: HashMap<String, Offset>,
    tree_changed: bool,
    needs_recompose: bool,
    draw_order_stamp: u64,
    current_offset_x: f32,
    current_offset_y: f32,
}

impl UIContext {
    pub fn new() -> Self {
        Self {
            page_id: String::new(),
            compose_stamp: 0,
            nodes: HashMap::new(),
            order: Vec::new(),
            draw_order: Vec::new(),
            clip_stack: Vec::new(),
            offset_stack: Vec::new(),
            owned_layouts: Vec::new(),
            layout_stack: Vec::new(),
            base_context_offset: HashMap::new(),
            tree_changed: false,
            needs_recompose: false,
            draw_order_stamp: 0,
            current_offset_x: 0.0,
            current_offset_y: 0.0,
        }
    }

    /// Begin a compose pass for the given page.
    pub fn begin(&mut self, page_id: &str) {
        self.page_id = page_id.to_string();
        self.compose_stamp += 1;
        self.order.clear();
        self.draw_order.clear();
        self.clip_stack.clear();
        self.offset_stack.clear();
        self.owned_layouts.clear();
        self.layout_stack.clear();
        self.base_context_offset.clear();
        self.current_offset_x = 0.0;
        self.current_offset_y = 0.0;
        self.tree_changed = false;
        self.needs_recompose = false;
    }

    /// End a compose pass: prune dead nodes and build draw order.
    pub fn end(&mut self) {
        // Remove nodes not composed in this stamp.
        let stamp = self.compose_stamp;
        self.nodes.retain(|_, node| node.composed_in(stamp));

        // Build draw order sorted by (render_layer, z_index, compose_order).
        self.draw_order = self.order.clone();
        let nodes = &self.nodes;
        self.draw_order.sort_by(|a, b| {
            let na = nodes.get(a);
            let nb = nodes.get(b);
            match (na, nb) {
                (Some(node_a), Some(node_b)) => {
                    let la = node_a.primitive().render_layer as u8;
                    let lb = node_b.primitive().render_layer as u8;
                    la.cmp(&lb).then(node_a.primitive().z_index.cmp(&node_b.primitive().z_index))
                }
                _ => std::cmp::Ordering::Equal,
            }
        });
        self.draw_order_stamp = self.compose_stamp;
    }

    /// Run per-frame update on all composed nodes.
    pub fn update(&mut self, state: &crate::state::UIState) {
        for key in &self.order {
            if let Some(node) = self.nodes.get_mut(key) {
                node.update(state);
            }
        }
    }

    /// Draw all visible nodes in draw order.
    pub fn draw(&self, renderer: &mut crate::renderer::renderer::Renderer) {
        for key in &self.draw_order {
            if let Some(node) = self.nodes.get(key) {
                if node.primitive().visible {
                    node.draw(renderer);
                }
            }
        }
    }

    /// Whether any node requires continuous updates (running animations).
    pub fn wants_continuous_update(&self) -> bool {
        self.nodes.values().any(|n| n.wants_continuous_update())
    }

    /// Push a clip rectangle onto the clip stack.
    pub fn push_clip(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.clip_stack.push(UIClipRect { x, y, width, height });
    }

    /// Pop the most recent clip rectangle.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Push an offset onto the offset stack, affecting subsequent node context offsets.
    pub fn push_offset(&mut self, x: f32, y: f32) {
        self.offset_stack.push(Offset { x, y });
        self.current_offset_x += x;
        self.current_offset_y += y;
    }

    /// Pop the most recent offset from the offset stack.
    pub fn pop_offset(&mut self) {
        if let Some(off) = self.offset_stack.pop() {
            self.current_offset_x -= off.x;
            self.current_offset_y -= off.y;
        }
    }

    /// Mark all nodes' visual caches as dirty (e.g. after a theme change).
    pub fn mark_all_nodes_dirty(&mut self) {
        for node in self.nodes.values_mut() {
            node.primitive_mut(); // touch to mark dirty -- mirrors C++ forceComposeDirty
        }
    }

    /// Whether the node tree structure changed during this compose pass.
    pub fn tree_changed(&self) -> bool {
        self.tree_changed
    }

    /// Whether any node requested a recompose.
    pub fn needs_recompose(&self) -> bool {
        self.needs_recompose
    }

    /// The current compose stamp.
    pub fn compose_stamp(&self) -> u64 {
        self.compose_stamp
    }

    /// The current page identifier.
    pub fn page_id(&self) -> &str {
        &self.page_id
    }

    /// Number of nodes currently retained.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Current context offset X (sum of all pushed offsets).
    pub fn current_offset_x(&self) -> f32 {
        self.current_offset_x
    }

    /// Current context offset Y (sum of all pushed offsets).
    pub fn current_offset_y(&self) -> f32 {
        self.current_offset_y
    }

    /// Get immutable access to a node by key.
    pub fn get_node(&self, key: &str) -> Option<&dyn UINode> {
        self.nodes.get(key).map(|n| n.as_ref())
    }

    /// Get mutable access to a node by key.
    pub fn get_node_mut(&mut self, key: &str) -> Option<&mut dyn UINode> {
        self.nodes.get_mut(key).map(|n| n.as_mut())
    }

    /// Apply the current context (offset, clip) to a node's primitive.
    pub fn apply_current_context(&mut self, key: &str) {
        if let Some(node) = self.nodes.get_mut(key) {
            let prim = node.primitive_mut();
            prim.context_offset_x = self.current_offset_x;
            prim.context_offset_y = self.current_offset_y;

            if self.clip_stack.is_empty() || !prim.clip_to_parent {
                prim.has_clip_rect = false;
                prim.clip_rect = UIClipRect::default();
            } else {
                prim.has_clip_rect = true;
                prim.clip_rect = *self.clip_stack.last().unwrap();
            }
        }

        self.base_context_offset.insert(
            key.to_string(),
            Offset {
                x: self.current_offset_x,
                y: self.current_offset_y,
            },
        );
    }

    /// Build a full key from the current page id and a local id.
    pub fn full_key(&self, id: &str) -> String {
        if self.page_id.is_empty() {
            id.to_string()
        } else {
            format!("{}.{}", self.page_id, id)
        }
    }

    /// Insert or retrieve a node, beginning its compose pass. Returns the full key.
    pub fn acquire_node(&mut self, full_key: String, factory: impl FnOnce(String) -> Box<dyn UINode>) -> String {
        let stamp = self.compose_stamp;

        let needs_insert = !self.nodes.contains_key(&full_key);

        if needs_insert {
            self.tree_changed = true;
            let mut node = factory(full_key.clone());
            node.begin_compose(stamp);
            self.nodes.insert(full_key.clone(), node);
            self.order.push(full_key.clone());
            self.apply_current_context(&full_key);
            return full_key;
        }

        let node = self.nodes.get_mut(&full_key).unwrap();
        if !node.composed_in(stamp) {
            node.begin_compose(stamp);
            self.order.push(full_key.clone());
        }
        self.apply_current_context(&full_key);
        full_key
    }

    /// Finalize a node build: finish compose and register with layout if active.
    pub fn finalize_build(&mut self, key: &str, info: &LayoutBuildInfo) {
        if let Some(node) = self.nodes.get_mut(key) {
            node.finish_compose();
        }

        // If there is an active layout, register this node as a child.
        if let Some(&layout_idx) = self.layout_stack.last() {
            self.owned_layouts[layout_idx].children.push(LayoutItem {
                node_key: key.to_string(),
                container_index: None,
                build: info.clone(),
            });
        }
    }

    // -- Layout API --

    /// Create a new layout container with the given flex direction.
    pub fn create_layout(&mut self, direction: FlexDirection) -> usize {
        let idx = self.owned_layouts.len();
        self.owned_layouts.push(LayoutState::new(direction));
        idx
    }

    /// Begin composing children into the layout at the given index.
    pub fn begin_layout(&mut self, layout_index: usize) {
        self.layout_stack.push(layout_index);
    }

    /// End composing children and resolve the layout.
    pub fn end_layout(&mut self, layout_index: usize) {
        if let Some(&top) = self.layout_stack.last() {
            if top == layout_index {
                self.layout_stack.pop();
            }
        }
        self.resolve_layout(layout_index);
    }

    /// Set layout properties on an existing layout container.
    pub fn layout_set_direction(&mut self, idx: usize, dir: FlexDirection) {
        if idx < self.owned_layouts.len() {
            self.owned_layouts[idx].direction = dir;
        }
    }

    pub fn layout_set_gap(&mut self, idx: usize, gap: f32) {
        if idx < self.owned_layouts.len() {
            self.owned_layouts[idx].gap = gap.max(0.0);
        }
    }

    pub fn layout_set_padding(&mut self, idx: usize, left: f32, top: f32, right: f32, bottom: f32) {
        if idx < self.owned_layouts.len() {
            let l = &mut self.owned_layouts[idx];
            l.padding_left = left.max(0.0);
            l.padding_top = top.max(0.0);
            l.padding_right = right.max(0.0);
            l.padding_bottom = bottom.max(0.0);
        }
    }

    pub fn layout_build_mut(&mut self, idx: usize) -> Option<&mut LayoutBuildInfo> {
        self.owned_layouts.get_mut(idx).map(|l| &mut l.build)
    }

    // -- Layout resolution --

    fn resolve_layout(&mut self, layout_index: usize) {
        if layout_index >= self.owned_layouts.len() {
            return;
        }

        let layout = &self.owned_layouts[layout_index];
        let direction = layout.direction;
        let gap = layout.gap;
        let pad_l = layout.padding_left;
        let pad_t = layout.padding_top;
        let pad_r = layout.padding_right;
        let pad_b = layout.padding_bottom;

        // Determine container frame from layout build info.
        let container_x = if layout.build.has_x { layout.build.x } else { 0.0 };
        let container_y = if layout.build.has_y { layout.build.y } else { 0.0 };
        let container_w = if layout.build.has_width { layout.build.width } else { 0.0 };
        let container_h = if layout.build.has_height { layout.build.height } else { 0.0 };

        let inner_w = (container_w - pad_l - pad_r).max(0.0);
        let inner_h = (container_h - pad_t - pad_b).max(0.0);

        let available_main = match direction {
            FlexDirection::Row => inner_w,
            FlexDirection::Column => inner_h,
        };
        let available_cross = match direction {
            FlexDirection::Row => inner_h,
            FlexDirection::Column => inner_w,
        };

        // Collect children info.
        let children: Vec<(String, LayoutBuildInfo)> = self.owned_layouts[layout_index]
            .children
            .iter()
            .map(|c| (c.node_key.clone(), c.build.clone()))
            .collect();

        if children.is_empty() {
            return;
        }

        // First pass: measure fixed sizes and total flex.
        let mut total_fixed: f32 = 0.0;
        let mut total_flex: f32 = 0.0;
        let gap_total = gap * (children.len() as f32 - 1.0).max(0.0);

        for (_key, build) in &children {
            if build.flex > 0.0 {
                total_flex += build.flex;
            } else {
                let main_size = match direction {
                    FlexDirection::Row => {
                        build.margin_left
                            + (if build.has_width { build.width } else { 0.0 })
                            + build.margin_right
                    }
                    FlexDirection::Column => {
                        build.margin_top
                            + (if build.has_height { build.height } else { 0.0 })
                            + build.margin_bottom
                    }
                };
                total_fixed += main_size;
            }
        }

        let flex_space = (available_main - total_fixed - gap_total).max(0.0);

        // Second pass: assign positions.
        let mut cursor = match direction {
            FlexDirection::Row => container_x + pad_l,
            FlexDirection::Column => container_y + pad_t,
        };
        let cross_start = match direction {
            FlexDirection::Row => container_y + pad_t,
            FlexDirection::Column => container_x + pad_l,
        };

        for (i, (key, build)) in children.iter().enumerate() {
            let (margin_before, margin_after) = match direction {
                FlexDirection::Row => (build.margin_left, build.margin_right),
                FlexDirection::Column => (build.margin_top, build.margin_bottom),
            };
            let (cross_margin_before, _cross_margin_after) = match direction {
                FlexDirection::Row => (build.margin_top, build.margin_bottom),
                FlexDirection::Column => (build.margin_left, build.margin_right),
            };

            cursor += margin_before;

            let main_size = if build.flex > 0.0 && total_flex > 0.0 {
                (flex_space * build.flex / total_flex).max(0.0)
            } else {
                match direction {
                    FlexDirection::Row => {
                        if build.has_width { build.width } else { 0.0 }
                    }
                    FlexDirection::Column => {
                        if build.has_height { build.height } else { 0.0 }
                    }
                }
            };

            let cross_size = match direction {
                FlexDirection::Row => {
                    if build.has_height { build.height } else { available_cross }
                }
                FlexDirection::Column => {
                    if build.has_width { build.width } else { available_cross }
                }
            };

            let (node_x, node_y, node_w, node_h) = match direction {
                FlexDirection::Row => (
                    cursor,
                    cross_start + cross_margin_before,
                    main_size,
                    cross_size,
                ),
                FlexDirection::Column => (
                    cross_start + cross_margin_before,
                    cursor,
                    cross_size,
                    main_size,
                ),
            };

            // Apply resolved frame to the node's primitive.
            if let Some(node) = self.nodes.get_mut(key) {
                let prim = node.primitive_mut();
                if !build.has_x || build.flex > 0.0 {
                    prim.x = node_x;
                }
                if !build.has_y || build.flex > 0.0 {
                    prim.y = node_y;
                }
                prim.width = node_w;
                prim.height = node_h;
            }

            cursor += main_size + margin_after;
            if i < children.len() - 1 {
                cursor += gap;
            }
        }
    }

    /// Consume recompose requests from all nodes.
    pub fn consume_recompose_request(&mut self) -> bool {
        let mut any = false;
        for node in self.nodes.values_mut() {
            if node.cache_dirty() {
                any = true;
            }
        }
        any
    }

    /// The compose-order keys (insertion order for this frame).
    pub fn order(&self) -> &[String] {
        &self.order
    }

    /// The draw-order keys (sorted by layer, z-index).
    pub fn draw_order(&self) -> &[String] {
        &self.draw_order
    }

    // -- Take/Return helpers for Builder pattern --

    /// Take a node out of the HashMap for builder use. If it doesn't exist, create one.
    /// Returns (node, full_key).
    pub fn take_for_builder(
        &mut self,
        id: &str,
        factory: fn(String) -> Box<dyn UINode>,
    ) -> (Box<dyn UINode>, String) {
        let full_key = self.full_key(id);
        let stamp = self.compose_stamp;

        if let Some(mut node) = self.nodes.remove(&full_key) {
            if !node.composed_in(stamp) {
                node.begin_compose(stamp);
                self.order.push(full_key.clone());
            }
            // Apply context (offset/clip) — store in base_context_offset map
            self.base_context_offset.insert(
                full_key.clone(),
                Offset {
                    x: self.current_offset_x,
                    y: self.current_offset_y,
                },
            );
            (node, full_key)
        } else {
            self.tree_changed = true;
            let mut node = factory(full_key.clone());
            node.begin_compose(stamp);
            self.order.push(full_key.clone());
            self.base_context_offset.insert(
                full_key.clone(),
                Offset {
                    x: self.current_offset_x,
                    y: self.current_offset_y,
                },
            );
            (node, full_key)
        }
    }

    /// Return a node to the HashMap after builder use, applying context and finalizing.
    pub fn return_and_finalize(
        &mut self,
        key: String,
        mut node: Box<dyn UINode>,
        info: LayoutBuildInfo,
    ) {
        // Apply context offset and clip
        {
            let prim = node.primitive_mut();
            prim.context_offset_x = self.current_offset_x;
            prim.context_offset_y = self.current_offset_y;

            if self.clip_stack.is_empty() || !prim.clip_to_parent {
                prim.has_clip_rect = false;
                prim.clip_rect = UIClipRect::default();
            } else {
                prim.has_clip_rect = true;
                prim.clip_rect = *self.clip_stack.last().unwrap();
            }
        }

        node.finish_compose();

        // Register with active layout if any
        if let Some(&layout_idx) = self.layout_stack.last() {
            self.owned_layouts[layout_idx].children.push(LayoutItem {
                node_key: key.clone(),
                container_index: None,
                build: info,
            });
        }

        self.nodes.insert(key, node);
    }

    // -- Factory methods --

    pub fn label(&mut self, id: &str) -> LabelBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(LabelNode::new(k)));
        LabelBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn button(&mut self, id: &str) -> ButtonBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(ButtonNode::new(k)));
        ButtonBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn panel(&mut self, id: &str) -> PanelBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(PanelNode::new(k)));
        PanelBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn glass_panel(&mut self, id: &str) -> GlassPanelBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(GlassPanelNode::new(k)));
        GlassPanelBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn progress(&mut self, id: &str) -> ProgressBarBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(ProgressBarNode::new(k)));
        ProgressBarBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn slider(&mut self, id: &str) -> SliderBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(SliderNode::new(k)));
        SliderBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn segmented(&mut self, id: &str) -> SegmentedControlBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(SegmentedControlNode::new(k)));
        SegmentedControlBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn input(&mut self, id: &str) -> InputBoxBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(InputBoxNode::new(k)));
        InputBoxBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn combo(&mut self, id: &str) -> ComboBoxBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(ComboBoxNode::new(k)));
        ComboBoxBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }

    pub fn sidebar(&mut self, id: &str) -> SidebarBuilder {
        let (node, key) = self.take_for_builder(id, |k| Box::new(SidebarNode::new(k)));
        SidebarBuilder { ctx: self, node, key, layout_build: LayoutBuildInfo::default() }
    }
}
