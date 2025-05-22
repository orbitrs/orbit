// orbit-spec.md - Specification for the .orbit file format

# Orbit File Format Specification

## Overview
The `.orbit` file format is a unified component format for the Orbit UI framework, combining markup, styling, and Rust logic into a single file. It is inspired by single-file component formats from frameworks like Vue, Blazor, and Svelte.

## File Structure
An `.orbit` file consists of three main sections:

1. **Template Section** - Contains the HTML/XML markup for the component
2. **Style Section** - Contains the CSS styling for the component
3. **Script Section** - Contains the Rust code for the component

### Example

```orbit
<template>
  <div class="greeting">
    <h1>Hello, {{ name }}!</h1>
    <button @click="increment">Count: {{ count }}</button>
  </div>
</template>

<style>
.greeting {
  font-family: Arial, sans-serif;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

h1 {
  color: #0070f3;
}

button {
  background-color: #0070f3;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}
</style>

<code lang="rust">
use orbitrs::prelude::*;

pub struct Greeting {
    name: String,
    count: i32,
}

pub struct GreetingProps {
    pub name: String,
}

impl Props for GreetingProps {}

impl Component for Greeting {
    type Props = GreetingProps;
    
    fn new(props: Self::Props) -> Self {
        Self {
            name: props.name,
            count: 0,
        }
    }
    
    fn render(&self) -> String {
        // In a real implementation, this would be generated from the template
        format!("<div>Hello, {}! Count: {}</div>", self.name, self.count)
    }
}

impl Greeting {
    pub fn increment(&mut self) {
        self.count += 1;
    }
}
</script>
```

## Template Syntax

### Text Interpolation
Use `{{ expression }}` to interpolate Rust expressions into the template.

### Attributes
- `:attr="expression"` - Bind an attribute value to a Rust expression
- `@event="handler"` - Attach an event handler to an element

### Components
Components are referenced using their PascalCase names:

```html
<CounterComponent :initial="5" @updated="handleUpdate" />
```

### Conditionals
Use `v-if`, `v-else-if`, and `v-else` for conditional rendering:

```html
<div v-if="count > 0">Positive</div>
<div v-else-if="count < 0">Negative</div>
<div v-else>Zero</div>
```

### Loops
Use `v-for` for list rendering:

```html
<ul>
  <li v-for="item in items">{{ item.name }}</li>
</ul>
```

### Slots
Use `<slot>` to define content insertion points:

```html
<div class="card">
  <div class="card-header">
    <slot name="header">Default Header</slot>
  </div>
  <div class="card-body">
    <slot>Default Content</slot>
  </div>
</div>
```

## Style Scoping
Styles in the `<style>` section are automatically scoped to the component by default, applying component-specific CSS class prefixes.

Use `<style scoped="false">` to opt out of style scoping.

## Script Requirements

The script section must:

1. Define a public struct for the component
2. Implement the `Component` trait for the struct
3. Define a Props type and implement the `Props` trait for it
4. Implement the `new` and `render` methods from the `Component` trait

## Multi-file components
For larger components, you can split the `.orbit` file into multiple files:

- `Component.orbit` - Main component file
- `Component.orbit.rs` - Additional Rust logic
- `Component.orbit.css` - Additional CSS styles

## Compilation Process
The `.orbit` file is compiled to Rust code via the `orbiton` CLI tool or as part of the build process.

The template section is parsed and converted to a Rust implementation of the `render` method.
