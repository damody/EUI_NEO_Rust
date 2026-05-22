use std::collections::HashMap;
use crate::rect::RectFrame;

/// Cached rendering surface (FBO).
struct CachedSurface {
    fbo: u32,
    texture: u32,
    width: i32,
    height: i32,
    bounds: RectFrame,
    valid: bool,
}

/// Manages cached surfaces for dirty-rect optimization.
///
/// Panels that have not changed can be drawn once into an off-screen FBO and
/// then blitted from the cache on subsequent frames, avoiding redundant draw
/// calls. This matches the C++ Renderer's surface caching strategy.
pub struct SurfaceCache {
    surfaces: HashMap<String, CachedSurface>,
}

impl SurfaceCache {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
        }
    }

    /// Draw into a cached surface. If the cache entry exists and `dirty` is false,
    /// the cached texture is blitted directly. Otherwise, the `painter` callback is
    /// invoked while the FBO is bound, and the result is stored in the cache.
    pub fn draw_cached(
        &mut self,
        _key: &str,
        _bounds: &RectFrame,
        _dirty: bool,
        _painter: &dyn Fn(),
    ) {
        // TODO: look up or create CachedSurface for key
        // TODO: if dirty or !valid:
        //   - resize FBO if bounds changed
        //   - glBindFramebuffer, glClear
        //   - call painter()
        //   - glBindFramebuffer(0)
        //   - mark valid = true
        // TODO: blit cached texture to current framebuffer
    }

    /// Release a single cached surface by key.
    pub fn release(&mut self, key: &str) {
        // TODO: glDeleteFramebuffers, glDeleteTextures
        self.surfaces.remove(key);
    }

    /// Release all cached surfaces.
    pub fn release_all(&mut self) {
        // TODO: iterate and delete GL resources
        self.surfaces.clear();
    }
}
