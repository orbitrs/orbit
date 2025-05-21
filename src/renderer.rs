//! Renderer implementation for the Orbit UI framework

use std::{error::Error, fmt, sync::Arc};

use skia_safe::{
    gpu::{gl::Interface, Budgeted, SurfaceOrigin},
    Surface, M44,
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

/// Thread-safe renderer interface
pub trait Renderer: Send + 'static {
    /// Handle a renderer message
    fn handle_message(&mut self, msg: RendererMessage) -> RendererResult;
}

/// Skia renderer state  
struct SkiaState {
    /// Skia GPU context
    gr_context: skia_safe::gpu::DirectContext,

    /// Skia render surface
    surface: Surface,

    /// Current transform stack
    transform_stack: Vec<M44>,

    /// Current width
    width: i32,

    /// Current height
    height: i32,
}

/// Skia-based renderer implementation
pub struct SkiaRenderer {
    /// Renderer state
    state: Option<SkiaState>,
}

impl SkiaRenderer {
    /// Create a new Skia renderer
    pub fn new() -> Self {
        Self { state: None }
    }

    /// Initialize Skia state
    fn init_skia(&mut self, width: i32, height: i32) -> RendererResult {
        // Create Skia GL interface
        let interface = Interface::new_native().ok_or_else(|| {
            let err: Box<dyn std::error::Error + Send> = Box::new(RendererError::GlError(
                "Failed to create GL interface".to_string(),
            ));
            err
        })?;

        // Create Skia GPU context with proper error handling
        let mut gr_context =
            skia_safe::gpu::DirectContext::new_gl(interface, None).ok_or_else(|| {
                let err: Box<dyn std::error::Error + Send> = Box::new(RendererError::SkiaError(
                    "Failed to create GPU context".to_string(),
                ));
                err
            })?;

        // Create image info for the surface
        let image_info = skia_safe::ImageInfo::new(
            (width, height),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );

        // Create surface with proper error handling
        let mut surface = Surface::new_render_target(
            &mut gr_context,
            Budgeted::No,
            &image_info,
            None,
            SurfaceOrigin::BottomLeft,
            None,
            false,
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
}
