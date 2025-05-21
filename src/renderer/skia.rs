// Skia renderer implementation for the Orbit UI framework
use std::{error::Error, fmt, sync::Arc};

use skia_safe::{
    gpu::gl::FramebufferInfo,
    gpu::{gl::Interface, BackendRenderTarget, DirectContext, Protected, SurfaceOrigin},
    ColorType, Surface, M44,
};

use crate::component::Node;

/// A message sent to the renderer thread
#[derive(Clone)]
pub enum RendererMessage {
    /// Initialize with dimensions
    Init { width: i32, height: i32 },
    /// Begin frame
    BeginFrame,
    /// End frame
    EndFrame,
    /// Render node
    Render { node: Arc<Node> },
    /// Shutdown renderer
    Shutdown,
}

/// Custom error type for renderer errors
#[derive(Debug)]
pub enum RendererError {
    /// Skia API error
    SkiaError(String),
    /// OpenGL error
    GlError(String),
    /// Initialization error
    InitError(String),
    /// General error
    GeneralError(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RendererError::SkiaError(msg) => write!(f, "Skia error: {}", msg),
            RendererError::GlError(msg) => write!(f, "OpenGL error: {}", msg),
            RendererError::InitError(msg) => write!(f, "Initialization error: {}", msg),
            RendererError::GeneralError(msg) => write!(f, "Renderer error: {}", msg),
        }
    }
}

impl Error for RendererError {}

/// Result from renderer operations
pub type RendererResult = Result<(), Box<dyn std::error::Error + Send>>;

/// Skia renderer state
pub(crate) struct SkiaState {
    /// Skia GPU context
    pub(crate) gr_context: DirectContext,

    /// Skia render surface
    pub(crate) surface: Surface,

    /// Current transform stack
    pub(crate) transform_stack: Vec<M44>,

    /// Current width
    pub(crate) width: i32,

    /// Current height
    pub(crate) height: i32,
}

/// Skia-based renderer implementation
pub struct SkiaRenderer {
    /// Renderer state
    pub(crate) state: Option<SkiaState>,
}

// Explicitly implement Send for SkiaRenderer since we control the access to the state
unsafe impl Send for SkiaRenderer {}

impl SkiaRenderer {
    /// Create a new Skia renderer
    pub fn new() -> Self {
        Self { state: None }
    }

    /// Initialize Skia state
    pub(crate) fn init_skia(&mut self, width: i32, height: i32) -> RendererResult {
        // Create Skia GL interface
        let interface = Interface::new_native().ok_or_else(|| {
            let err: Box<dyn std::error::Error + Send> = Box::new(RendererError::GlError(
                "Failed to create GL interface".to_string(),
            ));
            err
        })?;

        // Create Skia GPU context - note the use of the recommended function
        #[allow(deprecated)]
        let mut gr_context = DirectContext::new_gl(interface, None).ok_or_else(|| {
            let err: Box<dyn std::error::Error + Send> = Box::new(RendererError::SkiaError(
                "Failed to create GPU context".to_string(),
            ));
            err
        })?;

        // Create a framebuffer info struct for GL backend
        let fb_info = FramebufferInfo {
            fboid: 0,       // Use the default framebuffer
            format: 0x8058, // GL_RGBA8 format
            protected: Protected::No,
        };

        // Create backend render target - note the use of the new API
        #[allow(deprecated)]
        let backend_render_target = BackendRenderTarget::new_gl(
            (width, height),
            1, // samples per pixel
            8, // stencil bits
            fb_info,
        );

        // Create surface - note the use of the recommended function
        #[allow(deprecated)]
        let surface = Surface::from_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .ok_or_else(|| {
            let err: Box<dyn std::error::Error + Send> = Box::new(RendererError::SkiaError(
                "Failed to create surface".to_string(),
            ));
            err
        })?;

        self.state = Some(SkiaState {
            gr_context,
            surface,
            transform_stack: vec![M44::new_identity()],
            width,
            height,
        });

        Ok(())
    }

    /// Push a transform onto the stack
    fn push_transform(&mut self, transform: M44) {
        if let Some(state) = &mut self.state {
            let current = state
                .transform_stack
                .last()
                .cloned()
                .unwrap_or_else(M44::new_identity);

            // Create a mutable copy of current, then apply the transform
            let mut combined = current;
            combined.pre_concat(&transform);
            state.transform_stack.push(combined);
        }
    }

    /// Pop transform from the stack
    fn pop_transform(&mut self) {
        if let Some(state) = &mut self.state {
            if state.transform_stack.len() > 1 {
                state.transform_stack.pop();
            }
        }
    }

    /// Get current transform
    fn current_transform(&self) -> M44 {
        self.state
            .as_ref()
            .and_then(|state| state.transform_stack.last().cloned())
            .unwrap_or_else(M44::new_identity)
    }

    /// Render a test circle
    pub fn draw_test_circle(&mut self) -> RendererResult {
        let state = match &mut self.state {
            Some(state) => state,
            None => return Err(Box::new(RendererError::GeneralError("Renderer not initialized".into()))),
        };
        
        let canvas = state.surface.canvas();
        
        // Create a blue-ish paint
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.3, 0.5, 0.8, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::PaintStyle::Fill);
        
        // Draw a circle in the center of the canvas
        let center_x = state.width as f32 / 2.0;
        let center_y = state.height as f32 / 2.0;
        let radius = state.width.min(state.height) as f32 / 4.0;
        
        canvas.draw_circle(
            skia_safe::Point::new(center_x, center_y),
            radius,
            &paint,
        );
        
        Ok(())
    }
}
