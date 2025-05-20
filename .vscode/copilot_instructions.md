# GitHub Copilot Instructions for orbit

## About This Project

This project, `orbit`, is part of the Orbit UI framework ecosystem.
- **Primary Language:** Rust
- **Core Focus:** The core Orbit UI framework, enabling cross-platform UI development (Web, Native, Embedded) using `.orbit` components.

Refer to the main `README.md` in the project root for detailed information on its architecture, goals, and specific conventions.

## Key Technologies & Concepts

- **Orbit Framework:** Understand the structure of `.orbit` files (markup, Rust logic, optional CSS/JS/HTML). Be aware of the dual renderer approach (Skia for standard UI, WGPU for advanced UI) and the hybrid rendering architecture.
- **Rust Best Practices:** Adhere to Rust idioms, error handling patterns, and module organization.
- **Project-Specific Conventions:** Pay attention to naming conventions (e.g., `example.orbit`, `example.orbit.rs`), file structures (core, parser, renderer, cli), and coding styles outlined in the project's `README.md`.
- **Cross-Platform Nature:** Orbit aims for web (WASM), native, and embedded targets. Keep this in mind for solutions.
- **Renderer Backends:** Skia (standard UI, WASM stability) and WGPU (advanced UI, 3D, custom shaders). Understand the `RendererBackend` trait abstraction and the compositor.
- **Orbiton CLI:** The CLI tool for managing Orbit projects (`new`, `build`, `dev`).

## When Assisting:

- **Consult READMEs:** Always check the `README.md` in the `orbit` project root before providing solutions.
- **Code Generation:**
    - For `.orbit` files, ensure correct syntax for templates, Rust blocks, and any associated style or script sections.
    - For Rust code, prioritize safety, performance, and clarity, keeping in mind the core, parser, and renderer modules.
- **Tooling:** Be aware of `orbiton` (the CLI tool) and `orbit-analyzer` (the static analysis tool) and their roles in the ecosystem.
- **Renderer Awareness:** If dealing with UI components or rendering logic, consider the implications for both Skia and WGPU backends, the compositor, and renderer-specific optimizations.

By following these guidelines, you can provide more accurate and helpful assistance for the `orbit` project.
