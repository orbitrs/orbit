//! Enhanced renderer module with performance optimizations and component integration

// Renderer modules
pub mod skia;
pub mod wgpu;

// Re-export renderer items
pub use skia::{RendererError, RendererMessage, RendererResult, SkiaRenderer};

use crate::component::{ComponentId, Node};
use std::collections::HashMap;

/// Types of renderers available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
    /// Skia-based renderer
    Skia,
    /// WGPU-based renderer
    Wgpu,
    /// WebGL-based renderer (for web)
    WebGL,
    /// Automatic selection based on platform
    Auto,
}

/// Render context with performance optimizations
#[derive(Debug, Default)]
pub struct RenderContext {
    /// Viewport dimensions
    pub viewport_width: u32,
    pub viewport_height: u32,
    /// Device pixel ratio for high-DPI displays
    pub device_pixel_ratio: f32,
    /// Whether to enable vsync
    pub vsync_enabled: bool,
    /// Frame rate target
    pub target_fps: u32,
    /// Component dirty tracking for selective re-rendering
    dirty_components: HashMap<ComponentId, bool>,
}

impl RenderContext {
    /// Create new render context with default settings
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            device_pixel_ratio: 1.0,
            vsync_enabled: true,
            target_fps: 60,
            dirty_components: HashMap::new(),
        }
    }

    /// Mark a component as dirty (needing re-render)
    pub fn mark_dirty(&mut self, component_id: ComponentId) {
        self.dirty_components.insert(component_id, true);
    }

    /// Check if a component is dirty
    pub fn is_dirty(&self, component_id: ComponentId) -> bool {
        self.dirty_components
            .get(&component_id)
            .copied()
            .unwrap_or(false)
    }

    /// Clear dirty flag for a component
    pub fn mark_clean(&mut self, component_id: ComponentId) {
        self.dirty_components.remove(&component_id);
    }

    /// Get all dirty components
    pub fn get_dirty_components(&self) -> Vec<ComponentId> {
        self.dirty_components.keys().copied().collect()
    }

    /// Clear all dirty flags
    pub fn clear_all_dirty(&mut self) {
        self.dirty_components.clear();
    }
}

/// Render statistics for performance monitoring
#[derive(Debug, Default, Clone)]
pub struct RenderStats {
    /// Number of frames rendered
    pub frame_count: u64,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f32,
    /// Current FPS
    pub current_fps: f32,
    /// Number of draw calls in last frame
    pub draw_calls: u32,
    /// Number of vertices rendered in last frame
    pub vertex_count: u32,
    /// Number of components rendered in last frame
    pub component_count: u32,
}

/// Enhanced renderer interface with performance monitoring
pub trait Renderer {
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), crate::Error> {
        Ok(()) // Default implementation does nothing
    }

    /// Render a component tree with context and performance optimizations
    fn render(&mut self, root: &Node, context: &mut RenderContext) -> Result<(), crate::Error>;

    /// Render only dirty components for performance
    fn render_selective(
        &mut self,
        root: &Node,
        context: &mut RenderContext,
        _dirty_components: &[ComponentId],
    ) -> Result<(), crate::Error> {
        // Default implementation falls back to full render
        self.render(root, context)
    }

    /// Flush any pending operations
    fn flush(&mut self) -> Result<(), crate::Error> {
        Ok(()) // Default implementation does nothing
    }

    /// Clean up resources
    fn cleanup(&mut self) -> Result<(), crate::Error> {
        Ok(()) // Default implementation does nothing
    }

    /// Get the renderer name
    fn name(&self) -> &str;

    /// Get render statistics
    fn get_stats(&self) -> RenderStats {
        RenderStats::default()
    }

    /// Reset render statistics
    fn reset_stats(&mut self) {}

    /// Set render quality/performance level
    fn set_quality_level(&mut self, _level: QualityLevel) -> Result<(), crate::Error> {
        Ok(()) // Default implementation does nothing
    }
}

/// Render quality levels for performance tuning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    /// Fastest rendering, lowest quality
    Performance,
    /// Balanced rendering and quality
    Balanced,
    /// Highest quality, slower rendering
    Quality,
}

/// Create a renderer of the specified type
pub fn create_renderer(renderer_type: RendererType) -> Result<Box<dyn Renderer>, crate::Error> {
    match renderer_type {
        RendererType::Skia => {
            let renderer = SkiaRenderer::new();
            Ok(Box::new(renderer))
        }
        RendererType::Wgpu => {
            #[cfg(feature = "wgpu")]
            {
                let renderer = futures::executor::block_on(wgpu::WgpuRenderer::new())?;
                Ok(Box::new(renderer))
            }
            #[cfg(not(feature = "wgpu"))]
            {
                Err(crate::Error::Renderer(
                    "WGPU renderer not supported in this build".to_string(),
                ))
            }
        }
        RendererType::WebGL => {
            #[cfg(feature = "web")]
            {
                // WebGL renderer is not yet implemented; this is a placeholder for future implementation
                // The actual renderer would be defined in a webgl.rs module and imported
                Err(crate::Error::Renderer(
                    "WebGL renderer not yet implemented".to_string(),
                ))
            }
            #[cfg(not(feature = "web"))]
            {
                Err(crate::Error::Renderer(
                    "WebGL renderer not supported in this build".to_string(),
                ))
            }
        }
        RendererType::Auto => {
            #[cfg(target_arch = "wasm32")]
            {
                #[cfg(feature = "web")]
                {
                    // WebGL renderer is not yet implemented; this is a placeholder for future implementation
                    return Err(crate::Error::Renderer(
                        "WebGL renderer not yet implemented".to_string(),
                    ));
                }
                #[cfg(not(feature = "web"))]
                {
                    return Err(crate::Error::Renderer(
                        "No supported renderer for web platform in this build".to_string(),
                    ));
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                #[cfg(feature = "wgpu")]
                {
                    let renderer = futures::executor::block_on(wgpu::WgpuRenderer::new())?;
                    Ok(Box::new(renderer))
                }
                #[cfg(not(feature = "wgpu"))]
                {
                    let renderer = SkiaRenderer::new();
                    Ok(Box::new(renderer))
                }
            }
        }
    }
}

/// Enhanced renderer composition for hybrid UIs with performance monitoring
pub struct CompositeRenderer {
    /// The 2D renderer (usually Skia)
    pub renderer_2d: Box<dyn Renderer>,

    /// The 3D renderer (usually WGPU)
    pub renderer_3d: Box<dyn Renderer>,

    /// Combined render statistics
    stats: RenderStats,
}

impl CompositeRenderer {
    /// Create a new composite renderer with the specified renderers
    pub fn new(renderer_2d: Box<dyn Renderer>, renderer_3d: Box<dyn Renderer>) -> Self {
        Self {
            renderer_2d,
            renderer_3d,
            stats: RenderStats::default(),
        }
    }

    /// Create a default composite renderer
    pub fn create_default() -> Result<Self, crate::Error> {
        let renderer_2d = create_renderer(RendererType::Skia)?;
        let renderer_3d = create_renderer(RendererType::Wgpu)?;

        Ok(Self {
            renderer_2d,
            renderer_3d,
            stats: RenderStats::default(),
        })
    }
}

impl Renderer for CompositeRenderer {
    fn render(&mut self, root: &Node, context: &mut RenderContext) -> Result<(), crate::Error> {
        let start_time = std::time::Instant::now();

        // First render 3D elements
        self.renderer_3d.render(root, context)?;

        // Then render 2D elements on top
        self.renderer_2d.render(root, context)?;

        // Update statistics
        let frame_time = start_time.elapsed().as_secs_f32() * 1000.0;
        self.stats.frame_count += 1;
        self.stats.avg_frame_time_ms = (self.stats.avg_frame_time_ms + frame_time) / 2.0;
        self.stats.current_fps = 1000.0 / frame_time;

        // Combine stats from both renderers
        let stats_2d = self.renderer_2d.get_stats();
        let stats_3d = self.renderer_3d.get_stats();
        self.stats.draw_calls = stats_2d.draw_calls + stats_3d.draw_calls;
        self.stats.vertex_count = stats_2d.vertex_count + stats_3d.vertex_count;
        self.stats.component_count = stats_2d.component_count + stats_3d.component_count;

        Ok(())
    }

    fn render_selective(
        &mut self,
        root: &Node,
        context: &mut RenderContext,
        dirty_components: &[ComponentId],
    ) -> Result<(), crate::Error> {
        // Try selective rendering on both renderers
        self.renderer_3d
            .render_selective(root, context, dirty_components)?;
        self.renderer_2d
            .render_selective(root, context, dirty_components)?;
        Ok(())
    }

    fn name(&self) -> &str {
        "Enhanced Composite Renderer"
    }

    fn get_stats(&self) -> RenderStats {
        self.stats.clone()
    }

    fn reset_stats(&mut self) {
        self.stats = RenderStats::default();
        self.renderer_2d.reset_stats();
        self.renderer_3d.reset_stats();
    }

    fn set_quality_level(&mut self, level: QualityLevel) -> Result<(), crate::Error> {
        self.renderer_2d.set_quality_level(level)?;
        self.renderer_3d.set_quality_level(level)?;
        Ok(())
    }
}
