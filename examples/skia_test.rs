// Test program for the orbit window system with Skia rendering
use orbitrs::platform::PlatformType;

fn main() -> Result<(), orbitrs::Error> {
    // Create desktop adapter
    let mut adapter = orbitrs::platform::create_adapter(PlatformType::Desktop);

    // Initialize the adapter (creates window)
    adapter.init()?;

    // Run the application loop with sample content
    adapter.run()?;

    // Cleanup (should not reach here due to event_loop.run() consuming event_loop)
    adapter.shutdown()?;

    Ok(())
}
