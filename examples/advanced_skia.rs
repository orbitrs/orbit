// Advanced Skia test with custom rendering
use orbitrs::platform::PlatformType;

fn main() -> Result<(), orbitrs::Error> {
    // Create desktop adapter
    let mut adapter = orbitrs::platform::create_adapter(PlatformType::Desktop);

    // This example shows how we would use the platform.rs API in practice
    adapter.init()?;

    // Run the app
    adapter.run()?;

    Ok(())
}
