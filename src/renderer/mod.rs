// Renderer module for the Orbit UI framework
//!
//! This module contains the rendering system for the Orbit UI framework.
//! It provides a common Renderer trait that can be implemented by different
//! rendering backends (Skia, WebGL, etc.).

// Re-export skia module items
mod skia;
pub use skia::{RendererError, RendererMessage, RendererResult, SkiaRenderer};

// No need for these imports here since we're using the actual Renderer trait
// They're already available in the skia module

/// Types of renderers available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
    /// Skia-based renderer
    Skia,
    /// WebGL-based renderer (for web)
    WebGL,
    /// Automatic selection based on platform
    Auto,
}

/// Renderer interface
pub trait Renderer: Send + 'static {
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), crate::Error>;

    /// Render content
    fn render(&mut self, content: String) -> Result<(), crate::Error>;

    /// Flush rendered content to screen
    fn flush(&mut self) -> Result<(), crate::Error>;

    /// Clean up resources
    fn cleanup(&mut self) -> Result<(), crate::Error>;
}

/// Create a renderer of the specified type
pub fn create_renderer(renderer_type: RendererType) -> Box<dyn Renderer> {
    match renderer_type {
        RendererType::Skia => Box::new(SkiaRenderer::new()),
        RendererType::WebGL => {
            #[cfg(feature = "web")]
            {
                Box::new(WebGLRenderer::new())
            }
            #[cfg(not(feature = "web"))]
            {
                eprintln!("WebGL renderer not supported in this build, falling back to Skia");
                Box::new(SkiaRenderer::new())
            }
        }
        RendererType::Auto => {
            #[cfg(target_arch = "wasm32")]
            {
                #[cfg(feature = "web")]
                return Box::new(WebGLRenderer::new());
                #[cfg(not(feature = "web"))]
                panic!("No supported renderer for web platform in this build");
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                Box::new(SkiaRenderer::new())
            }
        }
    }
}

#[cfg(feature = "web")]
pub struct WebGLRenderer {
    // Web-specific state
}

#[cfg(feature = "web")]
impl WebGLRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "web")]
impl Default for WebGLRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "web")]
impl Renderer for WebGLRenderer {
    fn init(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn render(&mut self, _content: String) -> Result<(), crate::Error> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }
}

impl Renderer for SkiaRenderer {
    fn init(&mut self) -> Result<(), crate::Error> {
        // Default size, will be resized as needed
        self.init_skia(800, 600)
            .map_err(|e| crate::Error::Renderer(format!("Failed to initialize Skia: {}", e)))
    }

    fn render(&mut self, content: String) -> Result<(), crate::Error> {
        // Get the state or return error
        let state = match &mut self.state {
            Some(state) => state,
            None => return Err(crate::Error::Renderer("Renderer not initialized".into())),
        };

        // Clear the canvas with a light gray color
        let canvas = state.surface.canvas();
        canvas.clear(skia_safe::Color4f::new(0.9, 0.9, 0.9, 1.0));

        // Try to parse time from content (if available)
        let time = if content.contains("time") {
            // Very simple parser - in a real app we'd use serde_json
            let time_start = content.find(":").map(|pos| pos + 1).unwrap_or(0);
            let time_end = content[time_start..]
                .find("}")
                .map(|pos| pos + time_start)
                .unwrap_or(content.len());
            content[time_start..time_end]
                .trim()
                .parse::<f32>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Draw an animated circle if time is provided, otherwise a static one
        if time > 0.0 {
            self.draw_animated_circle(time);
            return Ok(());
        } else {
            return self
                .draw_test_circle()
                .map_err(|e| crate::Error::Renderer(format!("Failed to draw test circle: {}", e)));
        }
    }

    fn flush(&mut self) -> Result<(), crate::Error> {
        // Get the state or return error
        let state = match &mut self.state {
            Some(state) => state,
            None => return Err(crate::Error::Renderer("Renderer not initialized".into())),
        };

        // Flush the GPU context only - Surface doesn't have a flush method in this version
        state.gr_context.flush(None);

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), crate::Error> {
        // Reset state to release resources
        self.state = None;

        Ok(())
    }
}
