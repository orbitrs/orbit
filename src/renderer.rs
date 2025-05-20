// Renderer trait and implementation for Orbit UI framework

use crate::{
    component::Node,
    events::MouseEvent,
    style::Style,
};
use skia_safe::{
    gpu::{Context, Budgeted},
    SurfaceOrigin,
    Color, Paint, Surface,
};
use std::{collections::HashMap, sync::Arc};

/// Trait defining renderer capabilities
pub trait Renderer: Send {
    /// Initialize the renderer with given dimensions
    fn init(&mut self, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>>;

    /// Render a component tree
    fn render(&mut self, root: &Node) -> Result<(), Box<dyn std::error::Error>>;

    /// Process a mouse event
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<(), Box<dyn std::error::Error>>;

    /// Clean up resources
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Skia-based renderer implementation
pub struct SkiaRenderer {
    /// Skia rendering context
    context: Option<DirectContext>,
    /// Skia surface for drawing
    surface: Option<Surface>,
    /// Current viewport dimensions
    width: i32,
    height: i32,
    /// Cache of compiled styles
    style_cache: HashMap<String, Arc<Style>>,
    /// Current transformation matrix stack
    transform_stack: Vec<skia_safe::Matrix>,
}

impl SkiaRenderer {
    /// Create a new Skia renderer
    pub fn new() -> Self {
        Self {
            context: None,
            surface: None,
            width: 800,
            height: 600,
            style_cache: HashMap::new(),
            transform_stack: vec![skia_safe::Matrix::identity()],
        }
    }

    /// Create a new Skia renderer with specified dimensions
    pub fn new_with_size(width: i32, height: i32) -> Self {
        Self {
            context: None,
            surface: None,
            width,
            height,
            style_cache: HashMap::new(),
            transform_stack: vec![skia_safe::Matrix::identity()],
        }
    }

    /// Initialize the Skia context and surface
    fn init_skia(&mut self) -> Result<(), crate::Error> {
        #[cfg(feature = "gl")]
        {
            // Use native GL interface for the current GL context
            let interface = skia_safe::gpu::gl::Interface::new_native()
                .ok_or_else(|| crate::Error::Render("Failed to create GL interface".into()))?;

            // Create the DirectContext using the recommended approach
            let context = skia_safe::gpu::direct_contexts::make_gl(interface, None)
                .ok_or_else(|| {
                    crate::Error::Render("Failed to create Skia GL context".into())
                })?;

            // Create frame buffer info
            let fb_info = skia_safe::gpu::gl::FramebufferInfo::from_fboid(0); // Default framebuffer

            // Create backend render target with the modern API
            let backend_target = skia_safe::gpu::backend_render_targets::make_gl(
                (self.width, self.height),
                0, // sample count
                8, // stencil bits
                fb_info,
            );

            // Create a surface to draw on using the updated GPU surfaces API
            let mut context_copy = context.clone();
            let surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
                &mut context_copy, // Use a mutable reference to a copy of the context
                &backend_target,
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                skia_safe::ColorType::RGBA8888,
                None,
                None,
            )
            .ok_or_else(|| crate::Error::Render("Failed to create Skia surface".into()))?;

            self.surface = Some(surface);
            self.context = Some(context);

            // We don't store the canvas directly anymore - we'll get it from surface when needed
        }

        #[cfg(not(feature = "gl"))]
        {
            // CPU-based rendering for fallback
            let info = skia_safe::ImageInfo::new_n32_premul(
                (self.width, self.height),
                None, // No special color space
            );

            let surface =
                skia_safe::Surface::new_raster(&info, None, None).ok_or_else(|| {
                    crate::Error::Render("Failed to create Skia raster surface".into())
                })?;

            self.surface = Some(surface);
            // For CPU rendering, we don't need a GPU context
            self.context = None;

            // We don't store the canvas directly anymore - we'll get it from surface when needed
        }

        Ok(())
    }

    /// Push a transformation matrix onto the stack
    fn push_transform(&mut self, transform: skia_safe::Matrix) {
        let current = self.transform_stack.last().unwrap_or(&skia_safe::Matrix::identity());
        self.transform_stack.push(current.concat(&transform));
    }

    /// Pop the current transformation matrix
    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    /// Get the current transformation matrix
    fn current_transform(&self) -> skia_safe::Matrix {
        self.transform_stack.last().cloned().unwrap_or_default()
    }

    /// Create a new surface with the current dimensions
    fn create_surface(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let context = self.context.get_or_insert_with(|| {
            let gl_context = skia_safe::gpu::gl::Interface::new_native().unwrap();
            DirectContext::new_gl(Some(gl_context), None).unwrap()
        });

        let info = skia_safe::ImageInfo::new(
            (self.width, self.height),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );

        let surface = Surface::new_render_target(
            context,
            Budgeted::Yes,
            &info,
            None,
            SurfaceOrigin::TopLeft,
            None,
            false,
        )
        .ok_or("Failed to create surface")?;

        self.surface = Some(surface);
        Ok(())
    }

    /// Render a single node in the component tree
    fn render_node(&mut self, canvas: &mut skia_safe::Canvas, node: &Node) -> Result<(), Box<dyn std::error::Error>> {
        match node {
            Node::Element { tag, attributes, children, .. } => {
                // Push transformation for this element if needed
                if let Some(transform) = self.calculate_transform(attributes) {
                    self.push_transform(transform);
                    canvas.set_matrix(&self.current_transform());
                }

                // Apply styles
                let paint = self.create_paint_from_attributes(attributes);

                // Render the element
                match tag.as_str() {
                    "div" | "span" => self.render_container(canvas, children, &paint)?,
                    "button" => self.render_button(canvas, children, attributes, &paint)?,
                    "img" => self.render_image(canvas, attributes, &paint)?,
                    // Add more element types as needed
                    _ => self.render_container(canvas, children, &paint)?,
                }

                // Pop transformation if we pushed one
                if attributes.contains_key("transform") {
                    self.pop_transform();
                    canvas.set_matrix(&self.current_transform());
                }
            }
            Node::Text(text) => {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_color(Color::BLACK);
                
                // TODO: Implement proper text layout
                canvas.draw_text(text, (0.0, 0.0), &paint);
            }
            Node::Component { instance, .. } => {
                // TODO: Implement component rendering
            }
        }
        Ok(())
    }

    /// Calculate transformation matrix from attributes
    fn calculate_transform(&self, attributes: &HashMap<String, String>) -> Option<skia_safe::Matrix> {
        // TODO: Parse and apply transformations (translate, rotate, scale, etc.)
        None
    }

    /// Create a paint object from attributes
    fn create_paint_from_attributes(&self, attributes: &HashMap<String, String>) -> Paint {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        // Apply style attributes
        if let Some(color) = attributes.get("color") {
            // TODO: Parse color string
            paint.set_color(Color::BLACK);
        }

        // TODO: Apply more style attributes (stroke, fill, etc.)

        paint
    }

    /// Render a container element
    fn render_container(&mut self, canvas: &mut skia_safe::Canvas, children: &[Node], paint: &Paint) -> Result<(), Box<dyn std::error::Error>> {
        // Render children
        for child in children {
            self.render_node(canvas, child)?;
        }
        Ok(())
    }

    /// Render a button element
    fn render_button(&mut self, canvas: &mut skia_safe::Canvas, children: &[Node], attributes: &HashMap<String, String>, paint: &Paint) -> Result<(), Box<dyn std::error::Error>> {
        // Draw button background
        let rect = skia_safe::Rect::new(0.0, 0.0, 100.0, 40.0); // TODO: Calculate proper dimensions
        canvas.draw_rect(rect, paint);

        // Render children (button content)
        for child in children {
            self.render_node(canvas, child)?;
        }
        Ok(())
    }

    /// Render an image element
    fn render_image(&mut self, canvas: &mut skia_safe::Canvas, attributes: &HashMap<String, String>, paint: &Paint) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(src) = attributes.get("src") {
            // TODO: Load and cache images
            // TODO: Draw image with proper dimensions and scaling
        }
        Ok(())
    }
}

// Implement the Renderer trait for SkiaRenderer
impl Renderer for SkiaRenderer {
    fn init(&mut self, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.width = width;
        self.height = height;
        // Initialize Skia rendering context
        self.init_skia().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn render(&mut self, root: &Node) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure we have a surface (initialize if needed)
        if self.surface.is_none() {
            self.init(self.width, self.height)?;
        }

        // Make sure we still have a surface after initialization
        if let Some(surface) = self.surface.as_mut() {
            let canvas = surface.canvas();
            canvas.clear(skia_safe::Color::WHITE);

            // Render the component tree
            self.render_node(canvas, root)?;
        } else {
            return Err(Box::new(crate::Error::Render("Failed to create surface".into())));
        }

        Ok(())
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<(), Box<dyn std::error::Error>> {
        // Process mouse events
        // TODO: Implement proper event handling with hit testing
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clean up Skia resources
        self.surface = None;
        self.context = None;

        Ok(())
    }
}

// Helper function to strip HTML tags (used for simple text rendering)
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
}

/// WGPU-based renderer implementation (optional)
#[cfg(feature = "wgpu")]
pub mod wgpu {
    use super::*;

    /// WGPU-based renderer
    pub struct WgpuRenderer {
        // WGPU rendering context and resources would be here
        width: i32,
        height: i32,
    }

    impl WgpuRenderer {
        /// Create a new WGPU renderer
        pub fn new() -> Self {
            Self {
                width: 800,
                height: 600,
            }
        }
    }

    impl Renderer for WgpuRenderer {
        fn init(&mut self, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
            self.width = width;
            self.height = height;
            // Initialize WGPU rendering context
            Ok(())
        }

        fn render(&mut self, root: &Node) -> Result<(), Box<dyn std::error::Error>> {
            // Render using WGPU
            // This would use the component tree to create WGPU drawing commands
            Ok(())
        }

        fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<(), Box<dyn std::error::Error>> {
            // Process mouse events
            Ok(())
        }

        fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            // Clean up WGPU resources
            Ok(())
        }
    }
}

/// Factory function to create the appropriate renderer based on configuration
pub fn create_renderer(renderer_type: RendererType) -> Box<dyn Renderer> {
    match renderer_type {
        RendererType::Skia => Box::new(SkiaRenderer::new()),
        #[cfg(feature = "wgpu")]
        RendererType::Wgpu => Box::new(wgpu::WgpuRenderer::new()),
        #[cfg(not(feature = "wgpu"))]
        RendererType::Wgpu => Box::new(SkiaRenderer::new()), // Fallback to Skia if WGPU not available
        RendererType::Auto => {
            // Logic to choose the best renderer based on the platform and application needs
            // For now, default to Skia
            Box::new(SkiaRenderer::new())
        }
    }
}

/// Types of renderers available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
    /// Skia-based renderer for 2D UI
    Skia,
    /// WGPU-based renderer for 3D and advanced UI
    Wgpu,
    /// Automatically choose the best renderer
    Auto,
}
