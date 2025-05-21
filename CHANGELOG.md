# Changelog

All notable changes to the Orbit UI Framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8] - 2023-05-21

### Added
- Support for specialized ARM64 builds with optional reqwest dependency
- API documentation for component model

### Fixed
- Cross-compilation issues for Linux ARM64 targets

### Performance
- Improved rendering performance with Skia backend

## [0.1.7] - 2023-04-18

### Added
- Support for HTML5 parsing with html5ever

### Fixed
- Window resizing issues on macOS

### Changed
- Updated winit to version 0.29

## [0.1.6] - 2023-03-05

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
- Parser issues with commas in templates
- Improved whitespace handling in templates
- Event handler detection with @ prefix
- Expression parsing with proper spacing around operators

## [0.1.0] - 2025-05-21
- Initial public release

[Unreleased]: https://github.com/orbitrs/orbitrs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/orbitrs/orbitrs/releases/tag/v0.1.0
