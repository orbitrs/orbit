# Orbit Framework

![CI Status](https://github.com/orbitrs/orbitrs/actions/workflows/ci.yml/badge.svg)
![Release Status](https://github.com/orbitrs/orbitrs/actions/workflows/release.yml/badge.svg)
[![codecov](https://codecov.io/gh/orbitrs/orbit/branch/main/graph/badge.svg?token=CODECOV_TOKEN)](https://codecov.io/gh/orbitrs/orbit)
[![crates.io](https://img.shields.io/crates/v/orbitrs.svg)](https://crates.io/crates/orbitrs)

## 🌌 Overview

**Orbit** is a Rust-first, cross-platform UI framework that enables building **Web**, **Native**, and **Embedded** applications from a unified, single-source component format: `.orbit`. Inspired by frameworks like Razor, Vue, and Blazor, Orbit combines declarative markup with Rust logic for high-performance, flexible, and maintainable UI development.

---

## 💡 Vision & Philosophy

### Vision

> Build once, run anywhere — combining Rust’s power with an intuitive component model.

### Philosophy

* **Rust-Centric:** Harness Rust’s safety, speed, and concurrency at the core.
* **Unified Components:** Combine markup, style, and Rust logic seamlessly in `.orbit` files.
* **Platform Agnostic:** Target browsers (WASM), desktops, and embedded systems with one framework.
* **Developer Experience:** Prioritize live reloading, type safety, and clear developer tooling.

---

## 📦 Orbit File Structure & Naming Conventions

Orbit uses a flexible file naming convention to support simple to advanced workflows.

| File Name            | Purpose                                          |
| -------------------- | ------------------------------------------------ |
| `example.orbit`      | Core component file mixing markup and Rust logic |
| `example.orbit.rs`   | Optional Rust logic extension/shared file        |
| `example.orbit.html` | Optional raw HTML fragment                       |
| `example.orbit.css`  | Optional raw CSS styling                         |
| `example.orbit.js`   | Optional raw JavaScript (interop, utilities)     |

This encourages modular development while maintaining the ability to have everything in a single file for simplicity.

---

## 🎯 Core Goals

* ✅ Unified single-source UI components with Rust integration
* ✅ Cross-platform support: Web (WASM), Native (WGPU), Embedded
* ✅ Syntax inspired by Blazor/Vue but fully Rust-native
* ✅ Support for CSR, SSR, and Hydration for flexible rendering modes
* ✅ Powerful developer tooling: CLI, hot reloading, static type checks

---

## 🖥️ Renderer Backends: Hybrid Approach

Orbit uses a hybrid rendering architecture that combines the strengths of both Skia and WGPU:

| Feature                 | Skia (Standard UI)           | WGPU (Advanced UI)             |
| ----------------------- | ---------------------------- | ------------------------------ |
| High-quality 2D UI      | ✅ Native vector graphics     | ⚠️ Requires abstraction        |
| Hardware-accelerated 3D | ❌ Not supported              | ✅ Native support              |
| Custom shaders          | ⚠️ Limited support            | ✅ Full control                |
| Future game engine path | ❌ Not suitable               | ✅ Fully extensible            |
| WASM support            | ✅ Stable and production-ready | ⚠️ Experimental but evolving  |
| Performance for UI      | ✅ Optimized for 2D           | ⚠️ Overhead for simple UI     |

**Our Hybrid Solution:**
Orbit leverages both rendering backends through a unified abstraction:

* **Skia** for standard UI components where vector quality and WASM stability are critical
* **WGPU** for advanced UI with 3D elements, custom shaders, and game engine capabilities

This approach allows Orbit to excel across different application domains while maintaining a consistent API for developers.

---

## 🛣️ Roadmap & Milestones

### 🚩 Milestone 1: MVP (v0.1)

* `.orbit` parser (template + style + Rust blocks)
* Template-to-Rust code compiler
* Orbiton CLI: `new`, `build`, `dev` commands
* Skia-based renderer for standard UI components
* WASM runtime support

### 🚩 Milestone 2: Hybrid Renderer Architecture (v0.3)

* Introduce `RendererBackend` trait abstraction
* Develop WGPU renderer for advanced UI scenarios
* Implement renderer compositor for combining outputs
* Add heuristics for automatic renderer selection
* Define component metadata for renderer preferences

### 🚩 Milestone 3: Advanced Rendering Capabilities (v1.0+)

* Optimize coordination between renderers
* Seamless transitions between 2D and 3D content
* Scene graph, lighting, camera, and 3D controls
* Integration with Rust game engines (Bevy, etc.)
* Enable hybrid rendering modes (SSR, CSR, hydration)

### 🚩 Milestone 4: Ecosystem & Developer Experience (v1.x)

* Orbit Playground (online editor)
* OrbitKit component library with renderer-specific optimizations
* Orbiton plugin architecture
* Comprehensive documentation and tutorials

---

## 🧪 Development Strategy

* **Language:** Rust
* **Syntax:** HTML-like markup with embedded Rust expressions
* **Renderers:** 
  * Skia for standard UI components (2D, text, forms)
  * WGPU for advanced UI elements (3D, shaders, animations)
* **Renderer Selection:** Automatic based on component needs, with manual override
* **Build Tools:** Custom transpiler with `cargo` integration
* **CLI:** Orbiton for project management, build, and dev server
* **Extensibility:** Modular architecture allowing new backends (embedded, mobile)

---

## 📂 Project Structure

```plaintext
orbit/
├── core/             # Runtime core: state, events, reactivity
├── parser/           # Orbit file parser and AST
├── renderer/         # Renderer implementations
│   ├── common/       # Shared renderer abstractions
│   ├── skia/         # Skia renderer for standard UI
│   ├── wgpu/         # WGPU renderer for advanced UI
│   └── compositor/   # Renderer output compositor
├── cli/              # Orbiton CLI
├── examples/         # Sample apps and demos
├── docs/             # Documentation
└── orbit-spec.md     # Syntax and semantics specification
```

---

## 🔮 Future Considerations

* Orbit Inspector: DevTools for component state & renderer visualization
* Declarative Animation System: High-level, unified API for Skia and WGPU animations
* Advanced Theming Engine: Dynamic themes, custom theme creation, design token integration
* Embedded targets: `no_std` with optimized Skia/WGPU backends
* Renderer-specific performance optimizations and benchmarking tools
* Orbit Studio: WYSIWYG GUI Builder with renderer preview options
* Additional rendering backends: Vello, WebGPU native, Vulkan, Metal
* Runtime renderer switching based on performance metrics
* Precompiled `.orbit` to WASM packages for easy npm distribution

---

## 📢 Final Notes

Orbit is more than a UI framework—it's a **Rust-native UI ecosystem** designed for high performance, safety, and developer joy.
By embracing a hybrid approach with both Skia and WGPU, Orbit provides the best tools for each use case while maintaining a unified API—creating the foundation for the next generation of Rust apps across web, desktop, embedded, and beyond.

> The Orbit has begun. 🛰️
