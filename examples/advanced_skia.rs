// Advanced Skia test with custom rendering
use orbitui::platform::PlatformType;

fn main() -> Result<(), orbitui::Error> {
    // Create desktop adapter
    let mut adapter = orbitui::platform::create_adapter(PlatformType::Desktop);

    // This example shows how we would use the platform.rs API in practice
    adapter.init()?;

    // Run the app
    adapter.run()?;

    Ok(())
}
