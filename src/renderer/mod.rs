//! Updated renderer module with WGPU support

// Renderer modules
pub mod skia;
pub mod wgpu;

// Re-export renderer items
pub use skia::{RendererError, RendererMessage, RendererResult, SkiaRenderer};

use crate::component::Node;

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

/// Renderer interface
pub trait Renderer {
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), crate::Error> {
        Ok(()) // Default implementation does nothing
    }

    /// Render a component tree
    fn render(&mut self, root: &Node) -> Result<(), crate::Error>;

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

/// Renderer composition for hybrid UIs
pub struct CompositeRenderer {
    /// The 2D renderer (usually Skia)
    pub renderer_2d: Box<dyn Renderer>,

    /// The 3D renderer (usually WGPU)
    pub renderer_3d: Box<dyn Renderer>,
}

impl CompositeRenderer {
    /// Create a new composite renderer with the specified renderers
    pub fn new(renderer_2d: Box<dyn Renderer>, renderer_3d: Box<dyn Renderer>) -> Self {
        Self {
            renderer_2d,
            renderer_3d,
        }
    }

    /// Create a default composite renderer
    pub fn default() -> Result<Self, crate::Error> {
        let renderer_2d = create_renderer(RendererType::Skia)?;
        let renderer_3d = create_renderer(RendererType::Wgpu)?;

        Ok(Self {
            renderer_2d,
            renderer_3d,
        })
    }
}

impl Renderer for CompositeRenderer {
    fn render(&mut self, root: &Node) -> Result<(), crate::Error> {
        // First render 3D elements
        self.renderer_3d.render(root)?;

        // Then render 2D elements on top
        self.renderer_2d.render(root)?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Composite Renderer"
    }
}
