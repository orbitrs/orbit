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

    use glium::Surface;
    use glutin::context::ContextBuilder;
    use glutin::surface::WindowSurface;
    use winit::dpi::LogicalSize;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::window::WindowBuilder;

    /// Desktop platform adapter
    pub struct DesktopAdapter {
        // Desktop-specific resources
        renderer: Box<dyn Renderer>,
        display: Option<glium::Display<WindowSurface>>,
        event_loop: Option<EventLoop<()>>,
        running: bool,
    }

    impl DesktopAdapter {
        /// Create a new desktop adapter
        pub fn new() -> Self {
            Self {
                renderer: crate::renderer::create_renderer(RendererType::Auto),
                display: None,
                event_loop: None,
                running: false,
            }
        }

        /// Create a new desktop adapter with a specific renderer
        pub fn new_with_renderer(renderer_type: RendererType) -> Self {
            Self {
                renderer: crate::renderer::create_renderer(renderer_type),
                display: None,
                event_loop: None,
                running: false,
            }
        }

        /// Initialize the window
        fn init_window(&mut self) -> Result<(), crate::Error> {
            // Create an event loop
            let event_loop = EventLoop::new().map_err(|e| {
                crate::Error::Platform(format!("Failed to create event loop: {}", e))
            })?;

            // Window configuration
            let window_builder = WindowBuilder::new()
                .with_title("Orbit UI Application")
                .with_inner_size(LogicalSize::new(800.0, 600.0));

            // Context configuration
            let context_builder = ContextBuilder::new()
                .with_vsync(true)
                .with_hardware_acceleration(Some(true));

            // Create a display using simplified API
            let display = unsafe {
                glutin_winit::finalize_display(window_builder, context_builder, &event_loop)
                    .map_err(|e| {
                        crate::Error::Platform(format!("Failed to create display: {}", e))
                    })?
            };

            self.display = Some(display);
            self.event_loop = Some(event_loop);

            Ok(())
        }
    }

    impl PlatformAdapter for DesktopAdapter {
        fn init(&mut self) -> Result<(), crate::Error> {
            // Initialize desktop platform
            self.init_window()?;
            self.renderer.init()?;

            Ok(())
        }

        fn run(&mut self) -> Result<(), crate::Error> {
            // Set flag indicating we're running
            self.running = true;

            // Get the event loop
            let event_loop = match self.event_loop.take() {
                Some(el) => el,
                None => return Err(crate::Error::Platform("Event loop not initialized".into())),
            };

            // Sample root component to render
            let root_html = r#"<div class="root">
                <h1>Orbit UI Framework</h1>
                <p>Hello from the desktop platform!</p>
            </div>"#
                .to_string();

            // Get a reference to the display
            let display = match &self.display {
                Some(display) => display,
                None => return Err(crate::Error::Platform("Display not initialized".into())),
            };

            // Keep a reference to the renderer for the closure
            let renderer = &mut self.renderer;

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
                        let mut target = display.draw();
                        target.clear_color(0.2, 0.2, 0.2, 1.0);

                        // Render the UI
                        if let Err(e) = renderer.render(root_html.clone()) {
                            eprintln!("Rendering error: {}", e);
                        }

                        // Flush changes
                        if let Err(e) = renderer.flush() {
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
            self.renderer.cleanup()?;

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
