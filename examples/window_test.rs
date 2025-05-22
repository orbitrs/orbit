// Test program for the orbit window system
use orbit::platform::{create_adapter, PlatformType};

fn main() -> Result<(), orbit::Error> {
    // Create desktop adapter
    let mut adapter = create_adapter(PlatformType::Desktop);

    // Initialize the adapter (creates window)
    adapter.init()?;

    // Run the application loop
    adapter.run()?;

    // Cleanup (should not reach here due to event_loop.run() consuming event_loop)
    adapter.shutdown()?;

    Ok(())
}
