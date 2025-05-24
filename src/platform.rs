// Platform adapters for the Orbit UI framework

/// Trait for platform adapters
pub trait PlatformAdapter {
    /// Initialize the platform adapter
    fn init(&mut self) -> Result<(), crate::Error>;

    /// Start the main application loop
    fn run(&mut self) -> Result<(), crate::Error>;

    /// Shutdown the platform adapter
    fn shutdown(&mut self) -> Result<(), crate::Error>;
}

/// WebAssembly platform adapter
#[cfg(feature = "web")]
pub mod web {
    use super::PlatformAdapter;
    // Remove unused import
    // use wasm_bindgen::prelude::*;

    /// WebAssembly platform adapter
    pub struct WebAdapter {
        // Web-specific resources
    }

    impl WebAdapter {
        /// Create a new WebAssembly adapter
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Default for WebAdapter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PlatformAdapter for WebAdapter {
        fn init(&mut self) -> Result<(), crate::Error> {
            // Initialize web platform
            Ok(())
        }

        fn run(&mut self) -> Result<(), crate::Error> {
            // Set up web event loop
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), crate::Error> {
            // Clean up web resources
            Ok(())
        }
    }
}

/// Desktop platform adapter
#[cfg(feature = "desktop")]
pub mod desktop {
    use super::PlatformAdapter;
    use crate::renderer::{Renderer, RendererType};

    // Add Rc and RefCell for shared mutable state
    use std::cell::RefCell;
    use std::rc::Rc;

    use glium::Surface;
    use glutin::config::GlConfig;
    use glutin::context::NotCurrentGlContext;
    use glutin::display::{GetGlDisplay, GlDisplay};
    use glutin::surface::WindowSurface;
    use glutin_winit::GlWindow;
    use winit::dpi::LogicalSize;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::window::WindowBuilder;

    /// Desktop platform adapter
    pub struct DesktopAdapter {
        // Desktop-specific resources
        renderer: Rc<RefCell<Box<dyn Renderer>>>, // Changed to Rc<RefCell<...>>
        display: Option<glium::Display<WindowSurface>>,
        event_loop: Option<EventLoop<()>>,
        running: bool,
        start_time: std::time::Instant,
    }

    impl DesktopAdapter {
        /// Create a new desktop adapter
        pub fn new() -> Self {
            let renderer_result = crate::renderer::create_renderer(RendererType::Auto);
            let renderer = match renderer_result {
                Ok(r) => r,
                Err(e) => {
                    // Fall back to Skia renderer if Auto selection fails
                    eprintln!("Failed to create Auto renderer: {e}, falling back to Skia");
                    crate::renderer::create_renderer(RendererType::Skia)
                        .expect("Failed to create fallback Skia renderer")
                }
            };

            Self {
                renderer: Rc::new(RefCell::new(renderer)), // Wrap in Rc<RefCell<...>>
                display: None,
                event_loop: None,
                running: false,
                start_time: std::time::Instant::now(),
            }
        }

        /// Create a new desktop adapter with a specific renderer
        pub fn new_with_renderer(renderer_type: RendererType) -> Self {
            let renderer_result = crate::renderer::create_renderer(renderer_type);
            let renderer = match renderer_result {
                Ok(r) => r,
                Err(e) => {
                    // Fall back to Skia renderer if requested renderer fails
                    eprintln!(
                        "Failed to create {renderer_type:?} renderer: {e}, falling back to Skia"
                    );
                    crate::renderer::create_renderer(RendererType::Skia)
                        .expect("Failed to create fallback Skia renderer")
                }
            };

            Self {
                renderer: Rc::new(RefCell::new(renderer)), // Wrap in Rc<RefCell<...>>
                display: None,
                event_loop: None,
                running: false,
                start_time: std::time::Instant::now(),
            }
        }

        /// Initialize the window
        fn init_window(&mut self) -> Result<(), crate::Error> {
            // Create an event loop
            let event_loop = EventLoop::new()
                .map_err(|e| crate::Error::Platform(format!("Failed to create event loop: {e}")))?;

            // Window configuration
            let window_builder = WindowBuilder::new()
                .with_title("Orbit UI Application")
                .with_inner_size(LogicalSize::new(800.0, 600.0));

            // Create the basic config for glutin - using default sensible values
            let template = glutin::config::ConfigTemplateBuilder::new()
                .with_alpha_size(8)
                .with_stencil_size(8)
                .with_depth_size(24);

            // Create a builder for the glutin-winit integration
            let display_builder =
                glutin_winit::DisplayBuilder::new().with_window_builder(Some(window_builder));

            // Build the configs with the event loop
            let (mut window, gl_config) = display_builder
                .build(&event_loop, template, |configs| {
                    // Choose the config with the maximum number of samples
                    configs
                        .reduce(|accum, config| {
                            let transparency_check =
                                config.supports_transparency().unwrap_or(false)
                                    & !accum.supports_transparency().unwrap_or(false);

                            if transparency_check || config.num_samples() > accum.num_samples() {
                                config
                            } else {
                                accum
                            }
                        })
                        .unwrap()
                })
                .map_err(|e| crate::Error::Platform(format!("Failed to build display: {e:?}")))?;

            // Set up the context attributes with defaults
            let context_attribs = glutin::context::ContextAttributesBuilder::new().build(None);

            // Create the surfaced context - this handles a lot of the compatibility issues
            // and is safer than manually creating all the pieces
            let (gl_context, gl_surface) = match window.take() {
                Some(window) => {
                    // No need to get window size, it's handled automatically by build_surface_attributes

                    // Create the GL context
                    let not_current_context = unsafe {
                        gl_config
                            .display()
                            .create_context(&gl_config, &context_attribs)
                            .map_err(|e| {
                                crate::Error::Platform(format!(
                                    "Failed to create GL context: {e:?}"
                                ))
                            })?
                    };

                    // Build surface attributes from the window
                    let attrs = window.build_surface_attributes(<_>::default());

                    // Create the surface from the window
                    let surface = unsafe {
                        gl_config
                            .display()
                            .create_window_surface(&gl_config, &attrs)
                            .map_err(|e| {
                                crate::Error::Platform(format!(
                                    "Failed to create window surface: {e:?}"
                                ))
                            })?
                    };

                    // Make the context current with the surface
                    let context = not_current_context.make_current(&surface).map_err(|e| {
                        crate::Error::Platform(format!("Failed to make context current: {:?}", e))
                    })?;

                    (context, surface)
                }
                None => {
                    return Err(crate::Error::Platform("Window creation failed".into()));
                }
            };

            // Create the glium display from the context and surface
            let display =
                glium::Display::from_context_surface(gl_context, gl_surface).map_err(|e| {
                    crate::Error::Platform(format!("Failed to create Glium display: {:?}", e))
                })?;

            self.display = Some(display);
            self.event_loop = Some(event_loop);

            Ok(())
        }
    }

    impl Default for DesktopAdapter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PlatformAdapter for DesktopAdapter {
        fn init(&mut self) -> Result<(), crate::Error> {
            // Initialize desktop platform
            self.init_window()?;
            self.renderer.borrow_mut().init()?; // Use borrow_mut()

            Ok(())
        }

        fn run(&mut self) -> Result<(), crate::Error> {
            // Set flag indicating we're running
            self.running = true;
            let event_loop = match self.event_loop.take() {
                Some(el) => el,
                None => return Err(crate::Error::Platform("Event loop not initialized".into())),
            };

            // Get a clone of the display
            let display_clone = match &self.display {
                // Renamed for clarity
                Some(display) => display.clone(),
                None => return Err(crate::Error::Platform("Display not initialized".into())),
            };

            // Clone start_time for the closure (Instant is Copy)
            let start_time_clone = self.start_time; // Renamed for clarity

            // Clone the Rc for the renderer to move into the closure
            let renderer_rc = self.renderer.clone();

            // Run the event loop with the newer API
            let _ = event_loop.run(move |event, window_target| {
                window_target.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        window_target.exit();
                    }
                    Event::AboutToWait => {
                        // Draw a frame
                        let mut target = display_clone.draw(); // Use cloned display
                        target.clear_color(0.2, 0.2, 0.2, 1.0);

                        // Get the elapsed time for animation
                        let _elapsed = start_time_clone.elapsed().as_secs_f32(); // Use cloned start_time

                        // Create a dummy node for demonstration
                        let node = crate::component::Node::default();

                        // We should construct a proper node tree based on the elapsed time
                        // and component definitions, but for now we're just using a dummy node

                        // Create render context with default dimensions
                        let mut render_context = crate::renderer::RenderContext::new(800, 600);

                        // Render the UI
                        if let Err(e) = renderer_rc.borrow_mut().render(&node, &mut render_context)
                        {
                            // Use cloned Rc and borrow_mut()
                            eprintln!("Rendering error: {}", e);
                        }

                        // Flush changes
                        if let Err(e) = renderer_rc.borrow_mut().flush() {
                            // Use cloned Rc and borrow_mut()
                            eprintln!("Flush error: {}", e);
                        }

                        // Finish drawing
                        target.finish().unwrap();
                    }
                    _ => (),
                }
            });

            // We shouldn't reach here due to event_loop.run() consuming event_loop
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), crate::Error> {
            // Clean up desktop resources
            self.running = false;

            // Clean up renderer
            self.renderer.borrow_mut().cleanup()?; // Use borrow_mut()

            // Display is cleaned up automatically
            self.display = None;

            Ok(())
        }
    }
}

/// Factory function to create the appropriate platform adapter
pub fn create_adapter(platform_type: PlatformType) -> Box<dyn PlatformAdapter> {
    match platform_type {
        #[cfg(feature = "web")]
        PlatformType::Web => Box::new(web::WebAdapter::new()),
        #[cfg(not(feature = "web"))]
        PlatformType::Web => panic!("Web platform not supported in this build"),

        #[cfg(feature = "desktop")]
        PlatformType::Desktop => Box::new(desktop::DesktopAdapter::new()),
        #[cfg(not(feature = "desktop"))]
        PlatformType::Desktop => panic!("Desktop platform not supported in this build"),

        PlatformType::Auto => {
            // Logic to detect the current platform
            #[cfg(target_arch = "wasm32")]
            {
                #[cfg(feature = "web")]
                return Box::new(web::WebAdapter::new());
                #[cfg(not(feature = "web"))]
                panic!("Web platform not supported in this build");
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                #[cfg(feature = "desktop")]
                return Box::new(desktop::DesktopAdapter::new());
                #[cfg(not(feature = "desktop"))]
                panic!("Desktop platform not supported in this build");
            }
        }
    }
}

/// Types of platforms available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    /// Web platform (WASM)
    Web,
    /// Desktop platform (Windows, macOS, Linux)
    Desktop,
    /// Automatically detect platform
    Auto,
}
