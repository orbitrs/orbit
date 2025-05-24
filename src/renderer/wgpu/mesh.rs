//! 3D mesh handling for WGPU renderer

#[cfg(feature = "wgpu")]
use std::sync::Arc;
#[cfg(feature = "wgpu")]
use wgpu::{Buffer, BufferUsages, Device, Queue, VertexBufferLayout, VertexStepMode};

#[cfg(feature = "wgpu")]
use crate::Error;

/// A vertex with position and color
#[cfg(feature = "wgpu")]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Position in 3D space
    pub position: [f32; 3],
    /// RGB color
    pub color: [f32; 3],
}

impl Vertex {
    /// Create a new vertex with position and color
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }
    
    /// Get the vertex buffer layout for this vertex type
    pub fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// A 3D mesh with vertices and indices
pub struct Mesh {
    /// Vertex buffer
    pub vertex_buffer: Buffer,
    
    /// Index buffer
    pub index_buffer: Buffer,
    
    /// Number of indices
    pub num_indices: u32,
    
    /// Name of the mesh
    pub name: String,
}

impl Mesh {
    /// Create a new mesh from vertices and indices
    pub fn new(
        device: &Device,
        vertices: &[Vertex],
        indices: &[u16],
        name: &str,
    ) -> Result<Self, Error> {
        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        
        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", name)),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });
        
        Ok(Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            name: name.to_string(),
        })
    }
    
    /// Update the mesh vertices
    pub fn update_vertices(&self, queue: &Queue, vertices: &[Vertex]) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
    }
    
    /// Update the mesh indices
    pub fn update_indices(&mut self, queue: &Queue, indices: &[u16]) {
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));
        self.num_indices = indices.len() as u32;
    }
}

/// Predefined mesh shapes
pub struct MeshPrimitives;

impl MeshPrimitives {
    /// Create a cube mesh
    pub fn cube(device: &Device, size: f32) -> Result<Mesh, Error> {
        let vertices = [
            // Front face
            Vertex::new([-size, -size, size], [1.0, 0.0, 0.0]),
            Vertex::new([size, -size, size], [1.0, 0.0, 0.0]),
            Vertex::new([size, size, size], [1.0, 0.0, 0.0]),
            Vertex::new([-size, size, size], [1.0, 0.0, 0.0]),
            
            // Back face
            Vertex::new([-size, -size, -size], [0.0, 1.0, 0.0]),
            Vertex::new([-size, size, -size], [0.0, 1.0, 0.0]),
            Vertex::new([size, size, -size], [0.0, 1.0, 0.0]),
            Vertex::new([size, -size, -size], [0.0, 1.0, 0.0]),
            
            // Top face
            Vertex::new([-size, size, -size], [0.0, 0.0, 1.0]),
            Vertex::new([-size, size, size], [0.0, 0.0, 1.0]),
            Vertex::new([size, size, size], [0.0, 0.0, 1.0]),
            Vertex::new([size, size, -size], [0.0, 0.0, 1.0]),
            
            // Bottom face
            Vertex::new([-size, -size, -size], [1.0, 1.0, 0.0]),
            Vertex::new([size, -size, -size], [1.0, 1.0, 0.0]),
            Vertex::new([size, -size, size], [1.0, 1.0, 0.0]),
            Vertex::new([-size, -size, size], [1.0, 1.0, 0.0]),
            
            // Right face
            Vertex::new([size, -size, -size], [0.0, 1.0, 1.0]),
            Vertex::new([size, size, -size], [0.0, 1.0, 1.0]),
            Vertex::new([size, size, size], [0.0, 1.0, 1.0]),
            Vertex::new([size, -size, size], [0.0, 1.0, 1.0]),
            
            // Left face
            Vertex::new([-size, -size, -size], [1.0, 0.0, 1.0]),
            Vertex::new([-size, -size, size], [1.0, 0.0, 1.0]),
            Vertex::new([-size, size, size], [1.0, 0.0, 1.0]),
            Vertex::new([-size, size, -size], [1.0, 0.0, 1.0]),
        ];
        
        let indices: [u16; 36] = [
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,    // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];
        
        Mesh::new(device, &vertices, &indices, "Cube")
    }
    
    /// Create a sphere mesh
    pub fn sphere(device: &Device, radius: f32, segments: u32) -> Result<Mesh, Error> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate vertices
        for i in 0..=segments {
            let theta = i as f32 * std::f32::consts::PI / segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            for j in 0..=segments {
                let phi = j as f32 * 2.0 * std::f32::consts::PI / segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                
                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;
                
                let position = [x * radius, y * radius, z * radius];
                let color = [(x + 1.0) / 2.0, (y + 1.0) / 2.0, (z + 1.0) / 2.0];
                
                vertices.push(Vertex::new(position, color));
            }
        }
        
        // Generate indices
        for i in 0..segments {
            for j in 0..segments {
                let first = (i * (segments + 1)) + j;
                let second = first + segments + 1;
                
                indices.push(first as u16);
                indices.push(second as u16);
                indices.push((first + 1) as u16);
                
                indices.push(second as u16);
                indices.push((second + 1) as u16);
                indices.push((first + 1) as u16);
            }
        }
        
        Mesh::new(device, &vertices, &indices, "Sphere")
    }
    
    /// Create a plane mesh
    pub fn plane(device: &Device, size: f32) -> Result<Mesh, Error> {
        let vertices = [
            Vertex::new([-size, 0.0, -size], [0.5, 0.5, 0.5]),
            Vertex::new([size, 0.0, -size], [0.5, 0.5, 0.5]),
            Vertex::new([size, 0.0, size], [0.5, 0.5, 0.5]),
            Vertex::new([-size, 0.0, size], [0.5, 0.5, 0.5]),
        ];
        
        let indices = [0, 1, 2, 2, 3, 0];
        
        Mesh::new(device, &vertices, &indices, "Plane")
    }
}
