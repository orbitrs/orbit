// Test program for the orbit window system with Skia rendering
use orbit::platform::PlatformType;

fn main() -> Result<(), orbit::Error> {
    // Create desktop adapter
    let mut adapter = orbit::platform::create_adapter(PlatformType::Desktop);
    
    // Initialize the adapter (creates window)
    adapter.init()?;
    
    // Run the application loop with sample content
    adapter.run()?;
    
    // Cleanup (should not reach here due to event_loop.run() consuming event_loop)
    adapter.shutdown()?;
    
    Ok(())
}
