// Advanced Skia test with custom rendering
use orbit::platform::{PlatformAdapter, PlatformType};
use orbit::renderer::{Renderer, RendererType};
use std::time::{Duration, Instant};

fn main() -> Result<(), orbit::Error> {
    // Create desktop adapter
    let mut adapter = orbit::platform::create_adapter(PlatformType::Desktop);
    
    // This example shows how we would use the platform.rs API in practice
    adapter.init()?;
    
    // Run the app
    adapter.run()?;
    
    Ok(())
}
