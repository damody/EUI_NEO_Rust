use crate::types::RenderLayer;
use crate::color::Color;
use crate::rect::RectFrame;

/// FBO layer for multi-pass rendering.
struct LayerFBO {
    fbo: u32,
    texture: u32,
    width: i32,
    height: i32,
    bounds: RectFrame,
    needs_redraw: bool,
}

/// Manages the 4-layer FBO system (Backdrop / Content / Chrome / Popup).
///
/// Each layer is rendered to its own framebuffer object so that only dirty
/// layers need to be repainted. The final composite blits all layers in order
/// with alpha blending, matching the C++ Renderer layer compositing.
pub struct LayerManager {
    layers: [Option<LayerFBO>; 4],
}

impl LayerManager {
    pub fn new() -> Self {
        Self {
            layers: [None, None, None, None],
        }
    }

    /// Initialize FBOs for all layers at the given viewport size.
    pub fn init(&mut self, _width: i32, _height: i32) {
        // TODO: for each layer index 0..4:
        //   glGenFramebuffers, glGenTextures
        //   glBindTexture, glTexImage2D (RGBA, width, height)
        //   glFramebufferTexture2D
        //   store in self.layers[idx]
    }

    /// Resize all layer FBOs (call on window resize).
    pub fn resize(&mut self, _width: i32, _height: i32) {
        // TODO: recreate textures at new size, invalidate all
    }

    /// Set the content bounds for a layer.
    pub fn set_layer_bounds(&mut self, layer: RenderLayer, bounds: RectFrame) {
        let idx = layer as usize;
        if idx < 4 {
            if let Some(ref mut l) = self.layers[idx] {
                l.bounds = bounds;
            }
        }
    }

    /// Check whether a layer needs to be redrawn.
    pub fn needs_layer_redraw(&self, layer: RenderLayer) -> bool {
        let idx = layer as usize;
        self.layers
            .get(idx)
            .and_then(|l| l.as_ref())
            .map_or(true, |l| l.needs_redraw)
    }

    /// Bind the FBO for the given layer (all subsequent draws go into it).
    pub fn begin_layer(&mut self, _layer: RenderLayer) {
        // TODO: glBindFramebuffer(GL_FRAMEBUFFER, fbo)
        // TODO: glClear(GL_COLOR_BUFFER_BIT)
    }

    /// Unbind the current layer FBO (revert to default framebuffer).
    pub fn end_layer(&mut self) {
        // TODO: glBindFramebuffer(GL_FRAMEBUFFER, 0)
    }

    /// Composite all 4 layers onto the default framebuffer in order.
    pub fn composite_layers(&self, _background: &Color) {
        // TODO: glBindFramebuffer(GL_FRAMEBUFFER, 0)
        // TODO: clear with background color
        // TODO: enable blending
        // TODO: for each layer (Backdrop, Content, Chrome, Popup):
        //   bind layer texture, draw full-screen quad
    }

    /// Mark a single layer as needing redraw.
    pub fn invalidate_layer(&mut self, layer: RenderLayer) {
        let idx = layer as usize;
        if let Some(ref mut l) = self.layers[idx] {
            l.needs_redraw = true;
        }
    }

    /// Mark all layers as needing redraw.
    pub fn invalidate_all(&mut self) {
        for layer in &mut self.layers {
            if let Some(ref mut l) = layer {
                l.needs_redraw = true;
            }
        }
    }

    /// Release all FBO / texture resources.
    pub fn destroy(&mut self) {
        // TODO: glDeleteFramebuffers, glDeleteTextures for each layer
        for layer in &mut self.layers {
            *layer = None;
        }
    }
}
