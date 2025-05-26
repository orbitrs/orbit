// Beta compatibility test suite
// Tests specifically for beta toolchain compatibility

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// Utility function to run a command and return its output
fn run_command(command: &str, args: &[&str], working_dir: &str) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .current_dir(working_dir)
        .output()
        .map_err(|e| format!("Failed to execute command: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// Test that CI doesn't use desktop feature for orbiton
#[test]
#[ignore] // Only run when specifically requested
fn test_ci_orbiton_feature_usage() {
    let workspace_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let ci_path = Path::new(&workspace_dir)
        .join("../.github/workflows/ci.yml")
        .canonicalize()
        .expect("Failed to find CI workflow file");

    let ci_content = fs::read_to_string(ci_path).expect("Failed to read CI workflow file");

    assert!(
        !ci_content.contains("cargo test -p orbiton --features desktop"),
        "CI is using 'desktop' feature with orbiton, but this feature doesn't exist"
    );

    assert!(
        !ci_content.contains("cargo clippy -p orbiton --features desktop"),
        "CI is using 'desktop' feature with orbiton clippy, but this feature doesn't exist"
    );
}

// Test that beta clippy passes on orbit crate
#[test]
#[ignore] // Only run when specifically requested
fn test_beta_clippy_orbit() {
    let workspace_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let orbit_dir = Path::new(&workspace_dir)
        .join("../orbit")
        .canonicalize()
        .expect("Failed to find orbit directory");

    let result = run_command(
        "cargo",
        &[
            "+beta",
            "clippy",
            "--features",
            "desktop",
            "--",
            "-D",
            "warnings",
        ],
        orbit_dir.to_str().unwrap(),
    );

    assert!(
        result.is_ok(),
        "Beta clippy failed on orbit: {}",
        result.err().unwrap_or_default()
    );
}

// Test for uninlined format strings in orbit crate
#[test]
#[ignore] // Only run when specifically requested
fn test_uninlined_format_strings() {
    let workspace_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let orbit_src = Path::new(&workspace_dir)
        .join("../orbit/src")
        .canonicalize()
        .expect("Failed to find orbit/src directory");

    // Find files with potential format string issues using grep
    let result = run_command(
        "grep",
        &["-r", "format!(\"[^\"]*{}[^\"]*\", [^)]\\+)", "."],
        orbit_src.to_str().unwrap(),
    );

    if let Ok(output) = result {
        assert!(
            output.trim().is_empty(),
            "Found uninlined format strings in orbit crate: {output}"
        );
    }
}

// Test that all features used in CI workflows are valid
#[test]
#[ignore] // Only run when specifically requested
fn test_ci_feature_validity() {
    let workspace_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    // Package -> Features mapping
    let valid_features = [
        ("orbit", vec!["desktop", "web"]),
        ("orlint", vec![]),
        ("orbiton", vec!["local-dependencies", "ci"]),
        ("examples", vec!["desktop", "web"]),
    ];

    let ci_path = Path::new(&workspace_dir)
        .join("../.github/workflows/ci.yml")
        .canonicalize()
        .expect("Failed to find CI workflow file");

    let ci_content = fs::read_to_string(ci_path).expect("Failed to read CI workflow file");

    // Check each package-feature combination
    for (package, features) in valid_features.iter() {
        // Find all feature usages for this package
        let feature_regex =
            format!("cargo [\\w\\+]+ -p {package} [\\w\\s]+ --features ([\\w\\-]+)");
        let feature_regex_alt = format!("cargo [\\w\\+]+ --features ([\\w\\-]+) -p {package}");

        let feature_pattern = regex::Regex::new(&feature_regex).unwrap();
        let feature_pattern_alt = regex::Regex::new(&feature_regex_alt).unwrap();

        // Check all matches in the content
        for captures in feature_pattern.captures_iter(&ci_content) {
            if let Some(feature) = captures.get(1) {
                let feature_str = feature.as_str();
                assert!(
                    features.contains(&feature_str),
                    "CI uses invalid feature '{feature_str}' for package '{package}'"
                );
            }
        }

        for captures in feature_pattern_alt.captures_iter(&ci_content) {
            if let Some(feature) = captures.get(1) {
                let feature_str = feature.as_str();
                assert!(
                    features.contains(&feature_str),
                    "CI uses invalid feature '{feature_str}' for package '{package}'"
                );
            }
        }
    }
}

// Meta-test that ensures beta toolchain is installed
#[test]
fn test_beta_toolchain_available() {
    let result = run_command("rustup", &["toolchain", "list"], ".");

    match result {
        Ok(output) => {
            assert!(
                output.contains("beta"),
                "Beta toolchain not installed. Run 'rustup toolchain install beta' first."
            );
        }
        Err(e) => {
            panic!("Failed to check for beta toolchain: {e}");
        }
    }
}
