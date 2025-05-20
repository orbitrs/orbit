//! Renderer implementation for the Orbit UI framework

use std::{collections::HashMap, sync::{Arc, Mutex}};

use skia_safe::{
    gpu::{DirectContext, SurfaceOrigin, Budgeted, gl::Interface},
    Canvas, Color, M44, Paint, Surface,
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
    gr_context: DirectContext,
    
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
        Self {
            state: None
        }
    }
    
    /// Initialize Skia state
    fn init_skia(&mut self, width: i32, height: i32) -> RendererResult {
        // Create Skia GL interface
        let interface = Interface::new_native()?;
        
        // Create Skia GPU context
        let mut gr_context = DirectContext::make_gl(interface, None)?;
        
        // Create Skia surface
        let mut surface = Surface::make_render_target(
            &mut gr_context,
            Budgeted::No,
            &skia_safe::ImageInfo::new(
                (width, height),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None,
            ),
            1,
            Some(SurfaceOrigin::BottomLeft),
            None,
            false,
        ).ok_or("Failed to create surface")?;
        
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
            let current = state.transform_stack.last()
                .cloned()
                .unwrap_or_else(M44::new_identity);
                
            let combined = current.pre_concat(transform);
            state.transform_stack.push(combined);
        }
    }
    
    /// Pop transform from the stack
    fn pop_transform(&mut self) {
        if let Some(state) = &mut self.state {
            state.transform_stack.pop();
        }
    }
    
    /// Get current transform
    fn current_transform(&self) -> M44 {
        self.state.as_ref()
            .and_then(|state| state.transform_stack.last().cloned())
            .unwrap_or_else(M44::new_identity)
    }
}
