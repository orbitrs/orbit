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
    
    // Import glutin directly
    use glutin::event::{Event, WindowEvent};
    use glutin::event_loop::{ControlFlow, EventLoop};
    use glutin::window::WindowBuilder;
    use glutin::dpi::LogicalSize;
    use glutin::ContextBuilder;
    use glium::Surface;

    /// Desktop platform adapter
    pub struct DesktopAdapter {
        // Desktop-specific resources
        renderer: Box<dyn Renderer>,
        window: Option<glium::Display<glutin::surface::WindowSurface>>,
        event_loop: Option<EventLoop<()>>,
        running: bool,
    }

    impl DesktopAdapter {
        /// Create a new desktop adapter
        pub fn new() -> Self {
            Self {
                renderer: crate::renderer::create_renderer(RendererType::Auto),
                window: None,
                event_loop: None,
                running: false,
            }
        }

        /// Create a new desktop adapter with a specific renderer
        pub fn new_with_renderer(renderer_type: RendererType) -> Self {
            Self {
                renderer: crate::renderer::create_renderer(renderer_type),
                window: None,
                event_loop: None,
                running: false,
            }
        }

        /// Initialize the window
        fn init_window(&mut self) -> Result<(), crate::Error> {
            // Create an event loop
            let event_loop = EventLoop::new();

            // Window configuration
            let window_builder = WindowBuilder::new()
                .with_title("Orbit UI Application")
                .with_inner_size(LogicalSize::new(800.0, 600.0));

            // Context configuration
            let context_builder = ContextBuilder::new().with_vsync(true);

            // Create a display (window + context)
            let display = glium::Display::new(window_builder, context_builder, &event_loop)
                .map_err(|e| crate::Error::Platform(format!("Failed to create window: {}", e)))?;

            self.window = Some(display);
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

            // Run the event loop
            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Event::MainEventsCleared => {
                        // Render the UI
                        if let Err(e) = self.renderer.render(root_html.clone()) {
                            eprintln!("Rendering error: {}", e);
                        }

                        // Flush changes
                        if let Err(e) = self.renderer.flush() {
                            eprintln!("Flush error: {}", e);
                        }
                    }
                    _ => (),
                }
            });

            // We should never reach here as event_loop.run() is blocking
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), crate::Error> {
            // Clean up desktop resources
            self.running = false;

            // Clean up renderer
            self.renderer.cleanup()?;

            // Window is cleaned up automatically
            self.window = None;

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
