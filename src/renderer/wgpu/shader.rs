//! Shader management for WGPU renderer

#[cfg(feature = "wgpu")]
use std::sync::Arc;
#[cfg(feature = "wgpu")]
use wgpu::{ShaderModule, Device, ShaderModuleDescriptor, ShaderSource};

#[cfg(feature = "wgpu")]
use crate::Error;

/// Represents a compiled shader
#[cfg(feature = "wgpu")]
pub struct Shader {
    /// The compiled shader module
    module: ShaderModule,
    
    /// Shader name/label for debugging
    name: String,
    
    /// Entry point name
    entry_point: String,
}

impl Shader {
    /// Create a new shader from WGSL source
    pub fn from_wgsl(device: &Device, source: &str, name: &str, entry_point: &str) -> Result<Self, Error> {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(name),
            source: ShaderSource::Wgsl(source.into()),
        });
        
        Ok(Self {
            module,
            name: name.to_string(),
            entry_point: entry_point.to_string(),
        })
    }
    
    /// Create a new shader from SPIR-V binary
    pub fn from_spirv(device: &Device, spirv: &[u8], name: &str, entry_point: &str) -> Result<Self, Error> {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(name),
            source: ShaderSource::SpirV(std::borrow::Cow::Borrowed(spirv)),
        });
        
        Ok(Self {
            module,
            name: name.to_string(),
            entry_point: entry_point.to_string(),
        })
    }
    
    /// Get a reference to the shader module
    pub fn module(&self) -> &ShaderModule {
        &self.module
    }
    
    /// Get the shader name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the entry point name
    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }
}

/// A collection of shaders for different stages
pub struct ShaderSet {
    /// Vertex shader
    vertex: Option<Shader>,
    
    /// Fragment shader
    fragment: Option<Shader>,
    
    /// Compute shader
    compute: Option<Shader>,
}

impl ShaderSet {
    /// Create a new empty shader set
    pub fn new() -> Self {
        Self {
            vertex: None,
            fragment: None,
            compute: None,
        }
    }
    
    /// Set the vertex shader
    pub fn with_vertex(mut self, vertex: Shader) -> Self {
        self.vertex = Some(vertex);
        self
    }
    
    /// Set the fragment shader
    pub fn with_fragment(mut self, fragment: Shader) -> Self {
        self.fragment = Some(fragment);
        self
    }
    
    /// Set the compute shader
    pub fn with_compute(mut self, compute: Shader) -> Self {
        self.compute = Some(compute);
        self
    }
    
    /// Get the vertex shader
    pub fn vertex(&self) -> Option<&Shader> {
        self.vertex.as_ref()
    }
    
    /// Get the fragment shader
    pub fn fragment(&self) -> Option<&Shader> {
        self.fragment.as_ref()
    }
    
    /// Get the compute shader
    pub fn compute(&self) -> Option<&Shader> {
        self.compute.as_ref()
    }
}

/// Manages shader compilation and caching
pub struct ShaderManager {
    /// Device used for shader compilation
    device: Arc<Device>,
    
    /// Cached shaders
    shaders: std::collections::HashMap<String, Arc<Shader>>,
}

impl ShaderManager {
    /// Create a new shader manager
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            shaders: std::collections::HashMap::new(),
        }
    }
    
    /// Load a shader from WGSL source
    pub fn load_wgsl(&mut self, source: &str, name: &str, entry_point: &str) -> Result<Arc<Shader>, Error> {
        let key = format!("{}:{}", name, entry_point);
        
        if let Some(shader) = self.shaders.get(&key) {
            return Ok(shader.clone());
        }
        
        let shader = Arc::new(Shader::from_wgsl(&self.device, source, name, entry_point)?);
        self.shaders.insert(key, shader.clone());
        
        Ok(shader)
    }
    
    /// Load a shader from SPIR-V binary
    pub fn load_spirv(&mut self, spirv: &[u8], name: &str, entry_point: &str) -> Result<Arc<Shader>, Error> {
        let key = format!("{}:{}", name, entry_point);
        
        if let Some(shader) = self.shaders.get(&key) {
            return Ok(shader.clone());
        }
        
        let shader = Arc::new(Shader::from_spirv(&self.device, spirv, name, entry_point)?);
        self.shaders.insert(key, shader.clone());
        
        Ok(shader)
    }
    
    /// Create a basic shader set for 3D rendering
    pub fn create_basic_3d_shader_set(&mut self) -> Result<ShaderSet, Error> {
        // Basic vertex shader for 3D rendering
        let vertex_shader_src = r#"
            struct VertexInput {
                @location(0) position: vec3<f32>,
                @location(1) color: vec3<f32>,
            };
            
            struct VertexOutput {
                @builtin(position) clip_position: vec4<f32>,
                @location(0) color: vec3<f32>,
            };
            
            @group(0) @binding(0)
            var<uniform> transform: mat4x4<f32>;
            
            @vertex
            fn vs_main(in: VertexInput) -> VertexOutput {
                var out: VertexOutput;
                out.clip_position = transform * vec4<f32>(in.position, 1.0);
                out.color = in.color;
                return out;
            }
        "#;
        
        // Basic fragment shader
        let fragment_shader_src = r#"
            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return vec4<f32>(in.color, 1.0);
            }
        "#;
        
        let vertex = self.load_wgsl(vertex_shader_src, "Basic 3D Vertex", "vs_main")?;
        let fragment = self.load_wgsl(fragment_shader_src, "Basic 3D Fragment", "fs_main")?;
        
        Ok(ShaderSet::new()
            .with_vertex((*vertex).clone())
            .with_fragment((*fragment).clone()))
    }
    
    /// Clear the shader cache
    pub fn clear_cache(&mut self) {
        self.shaders.clear();
    }
}
