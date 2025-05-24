//! WGPU renderer base functionality

#[cfg(feature = "wgpu")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "wgpu")]
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};

#[cfg(feature = "wgpu")]
use crate::component::Node;
#[cfg(feature = "wgpu")]
use crate::renderer::Renderer;
#[cfg(feature = "wgpu")]
use crate::Error;

/// WGPU renderer for 3D rendering
#[cfg(feature = "wgpu")]
pub struct WgpuRenderer {
    /// WGPU instance
    instance: Instance,

    /// WGPU device
    device: Arc<Device>,

    /// WGPU queue
    queue: Arc<Queue>,

    /// WGPU surface
    surface: Option<Surface>,

    /// Surface configuration
    surface_config: Option<SurfaceConfiguration>,

    /// WGPU adapter
    adapter: Adapter,
}

#[cfg(feature = "wgpu")]
impl WgpuRenderer {
    /// Create a new WGPU renderer
    pub async fn new() -> Result<Self, Error> {
        // Create WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // Select adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| Error::Renderer("Failed to find GPU adapter".to_string()))?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Orbit WGPU Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| Error::Renderer(format!("Failed to create device: {}", e)))?;

        Ok(Self {
            instance,
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: None,
            surface_config: None,
            adapter,
        })
    }

    /// Configure the renderer with a surface
    pub fn configure_surface(
        &mut self,
        surface: Surface,
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        let surface_caps = surface.get_capabilities(&self.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&self.device, &config);

        self.surface = Some(surface);
        self.surface_config = Some(config);

        Ok(())
    }

    /// Get a reference to the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get a reference to the queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if let Some(surface) = &self.surface {
            if let Some(config) = &mut self.surface_config {
                config.width = width;
                config.height = height;
                surface.configure(&self.device, config);
                Ok(())
            } else {
                Err(Error::Renderer("Surface not configured".to_string()))
            }
        } else {
            Err(Error::Renderer("Surface not initialized".to_string()))
        }
    }
}

#[cfg(feature = "wgpu")]
impl Renderer for WgpuRenderer {
    fn render(&mut self, root: &Node) -> Result<(), Error> {
        // Get current surface texture
        if let Some(surface) = &self.surface {
            let frame = surface
                .get_current_texture()
                .map_err(|e| Error::Renderer(format!("Failed to get next frame: {}", e)))?;

            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            // Clear the frame
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                // Render components from the node tree would happen here
            }

            // Submit commands
            self.queue.submit(std::iter::once(encoder.finish()));
            frame.present();

            Ok(())
        } else {
            Err(Error::Renderer("Surface not initialized".to_string()))
        }
    }

    fn name(&self) -> &str {
        "WGPU Renderer"
    }
}
