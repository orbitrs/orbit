# Changelog

All notable changes to the Orbit UI Framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8] - 2025-05-26

### Added
- Support for specialized ARM64 builds with optional reqwest dependency
- API documentation for component model

### Fixed
- Cross-compilation issues for Linux ARM64 targets
- Compiler warnings in component implementation files
- Clippy lints with appropriate allow attributes

### Performance
- Improved rendering performance with Skia backend

## [0.1.7] - 2025-04-18

### Added
- Support for HTML5 parsing with html5ever

### Fixed
- Window resizing issues on macOS

### Changed
- Updated winit to version 0.29

## [0.1.6] - 2025-03-05

### Added
- Basic component model

### Fixed
- Text rendering with complex scripts

### Changed
- Updated skia-safe to 0.84.0

## [Unreleased]

### Added
- Initial implementation of orbit parser for `.orbit` files
- Basic component model with lifecycle methods
- Skia-based rendering engine
- Template syntax with expressions and event handling
- Styling system with scoped CSS

### Fixed
- WASM build configuration with proper feature flag separation
- CI/CD pipeline build errors on Linux systems
- Glutin compilation errors with targeted feature selection
- Missing pkg-config dependency in CI environment
- Parser issues with commas in templates
- Improved whitespace handling in templates
- Event handler detection with @ prefix
- Expression parsing with proper spacing around operators

### Build System
- **WASM Feature Separation**: Fixed workspace feature unification that was causing desktop dependencies to be included in WASM builds
- **Target-Specific Dependencies**: Implemented proper separation between desktop and web builds using target-specific dependency declarations
- **CI/CD Package-Specific Builds**: Updated build pipeline to compile specific packages for WASM rather than entire workspace

## [0.1.0] - 2025-05-21
- Initial public release

[Unreleased]: https://github.com/orbitrs/orbitrs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/orbitrs/orbitrs/releases/tag/v0.1.0
