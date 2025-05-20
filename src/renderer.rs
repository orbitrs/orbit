// Renderer module for the Orbit UI framework

/// Renderers in Orbit
pub trait Renderer {
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), crate::Error>;

    /// Render a component tree to the target
    fn render(&mut self, root_html: String) -> Result<(), crate::Error>;

    /// Flush pending changes to the target
    fn flush(&mut self) -> Result<(), crate::Error>;

    /// Clean up resources
    fn cleanup(&mut self) -> Result<(), crate::Error>;
}

/// Skia-based renderer implementation
pub mod skia {
    use super::Renderer;

    /// Skia-based renderer
    pub struct SkiaRenderer {
        // Skia rendering context and resources
        surface: Option<skia_safe::Surface>,
        // Don't directly store a canvas reference - we'll get it from surface when needed
        context: Option<skia_safe::gpu::DirectContext>,
        width: i32,
        height: i32,
    }

    impl SkiaRenderer {
        /// Create a new Skia renderer
        pub fn new() -> Self {
            Self {
                surface: None,
                context: None,
                width: 800,
                height: 600,
            }
        }

        /// Create a new Skia renderer with specified dimensions
        pub fn new_with_size(width: i32, height: i32) -> Self {
            Self {
                surface: None,
                context: None,
                width,
                height,
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
    }

    impl Renderer for SkiaRenderer {
        fn init(&mut self) -> Result<(), crate::Error> {
            // Initialize Skia rendering context
            self.init_skia()
        }

        fn render(&mut self, root_html: String) -> Result<(), crate::Error> {
            // Ensure we have a surface (initialize if needed)
            if self.surface.is_none() {
                self.init()?;
            }

            // Make sure we still have a surface after initialization
            if self.surface.is_none() {
                return Err(crate::Error::Render("Failed to create surface".into()));
            }

            // Create a new surface with a writable canvas
            // This is safer than trying to cast the immutable canvas reference to mutable
            let surface = self.surface.as_mut().unwrap();

            // We need to create a proper mutable canvas
            // One approach is to clear and draw at this level
            let canvas = surface.canvas();
            canvas.clear(skia_safe::Color::WHITE);

            // For rendering HTML, we need a helper that works with immutable canvas
            render_html_to_canvas_ref(canvas, &root_html)?;

            Ok(())
        }

        fn flush(&mut self) -> Result<(), crate::Error> {
            // Flush Skia rendering
            // If we're using GPU rendering, wait for the GPU to finish
            if let Some(context) = &mut self.context {
                // Context flush requires FlushInfo but it needs to be passed by reference
                let flush_info = skia_safe::gpu::FlushInfo::default();
                context.flush(&flush_info);
            }

            Ok(())
        }

        fn cleanup(&mut self) -> Result<(), crate::Error> {
            // Clean up Skia resources
            self.surface = None;
            self.context = None;

            Ok(())
        }
    }

    // Helper function to render HTML content - moved outside of impl to avoid borrow issues
    // Version for immutable reference to Canvas
    fn render_html_to_canvas_ref(
        canvas: &skia_safe::Canvas,
        html: &str,
    ) -> Result<(), crate::Error> {
        // This is a simplistic HTML renderer
        // In a real implementation, we would use a proper HTML parser like html5ever

        // Example of rendering text
        let paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 0.0, 0.0, 1.0), None);

        // Create a font with default typeface - get from FontMgr since Typeface::default() doesn't exist
        let font_mgr = skia_safe::FontMgr::default();
        let typeface = font_mgr
            .legacy_make_typeface(None, skia_safe::FontStyle::normal())
            .ok_or_else(|| crate::Error::Render("Failed to create default typeface".into()))?;
        let font = skia_safe::Font::new(typeface, 18.0);

        // Strip HTML tags for this simple example
        let text = strip_html_tags(html);

        // Draw text at position (50, 50)
        canvas.draw_str(&text, (50.0, 50.0), &font, &paint);

        Ok(())
    }

    // Helper function to strip HTML tags
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
}

/// WGPU-based renderer implementation (optional)
#[cfg(feature = "wgpu")]
pub mod wgpu {
    use super::Renderer;

    /// WGPU-based renderer
    pub struct WgpuRenderer {
        // WGPU rendering context and resources would be here
    }

    impl WgpuRenderer {
        /// Create a new WGPU renderer
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Renderer for WgpuRenderer {
        fn init(&mut self) -> Result<(), crate::Error> {
            // Initialize WGPU rendering context
            Ok(())
        }

        fn render(&mut self, root_html: String) -> Result<(), crate::Error> {
            // Render using WGPU
            // This would parse the HTML-like string and convert it to WGPU drawing commands
            println!("WGPU rendering HTML: {}", root_html);
            Ok(())
        }

        fn flush(&mut self) -> Result<(), crate::Error> {
            // Flush WGPU rendering
            Ok(())
        }

        fn cleanup(&mut self) -> Result<(), crate::Error> {
            // Clean up WGPU resources
            Ok(())
        }
    }
}

/// Factory function to create the appropriate renderer based on configuration
pub fn create_renderer(renderer_type: RendererType) -> Box<dyn Renderer> {
    match renderer_type {
        RendererType::Skia => Box::new(skia::SkiaRenderer::new()),
        #[cfg(feature = "wgpu")]
        RendererType::Wgpu => Box::new(wgpu::WgpuRenderer::new()),
        #[cfg(not(feature = "wgpu"))]
        RendererType::Wgpu => Box::new(skia::SkiaRenderer::new()), // Fallback to Skia if WGPU not available
        RendererType::Auto => {
            // Logic to choose the best renderer based on the platform and application needs
            // For now, default to Skia
            Box::new(skia::SkiaRenderer::new())
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
