//! Camera system for 3D rendering

#[cfg(feature = "wgpu")]
use std::f32::consts::PI;
#[cfg(feature = "wgpu")]
use wgpu::{Device, Buffer, BufferUsages};
#[cfg(feature = "wgpu")]
use cgmath::{Point3, Matrix4, perspective, Deg, Vector3, Quaternion, Rad};

#[cfg(feature = "wgpu")]
use crate::Error;

/// Uniform buffer for camera transform
#[cfg(feature = "wgpu")]
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// View projection matrix
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Create a new camera uniform
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }
    
    /// Update the view projection matrix
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

/// Camera for 3D rendering
pub struct Camera {
    /// Eye position
    pub position: Point3<f32>,
    
    /// Target position
    pub target: Point3<f32>,
    
    /// Up vector
    pub up: Vector3<f32>,
    
    /// Aspect ratio
    pub aspect: f32,
    
    /// Field of view in degrees
    pub fovy: f32,
    
    /// Near clipping plane
    pub znear: f32,
    
    /// Far clipping plane
    pub zfar: f32,
}

impl Camera {
    /// Create a new camera
    pub fn new(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }
    
    /// Build the view projection matrix
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.position, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
    
    /// Update aspect ratio
    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

/// Camera controller for interactive camera movement
pub struct CameraController {
    /// Camera being controlled
    camera: Camera,
    
    /// Camera speed
    speed: f32,
    
    /// Is looking around
    is_looking: bool,
    
    /// Movement flags
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    
    /// Rotation in euler angles
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl CameraController {
    /// Create a new camera controller
    pub fn new(camera: Camera, speed: f32) -> Self {
        let direction = camera.target - camera.position;
        let yaw = Rad(direction.x.atan2(direction.z));
        let pitch = Rad((direction.y / direction.magnitude()).asin());
        
        Self {
            camera,
            speed,
            is_looking: false,
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            yaw,
            pitch,
        }
    }
    
    /// Process keyboard input
    pub fn process_keyboard(&mut self, key: &str, pressed: bool) {
        match key {
            "w" | "ArrowUp" => self.forward = pressed,
            "s" | "ArrowDown" => self.backward = pressed,
            "a" | "ArrowLeft" => self.left = pressed,
            "d" | "ArrowRight" => self.right = pressed,
            "q" => self.up = pressed,
            "e" => self.down = pressed,
            _ => (),
        }
    }
    
    /// Process mouse movement
    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) {
        if self.is_looking {
            const LOOK_SENSITIVITY: f32 = 0.005;
            self.yaw += Rad(delta_x * LOOK_SENSITIVITY);
            self.pitch -= Rad(delta_y * LOOK_SENSITIVITY);
            
            // Clamp pitch to avoid the camera flipping
            if self.pitch < Rad(-PI / 2.0 + 0.1) {
                self.pitch = Rad(-PI / 2.0 + 0.1);
            }
            if self.pitch > Rad(PI / 2.0 - 0.1) {
                self.pitch = Rad(PI / 2.0 - 0.1);
            }
            
            // Update camera target based on rotation
            self.update_camera_vectors();
        }
    }
    
    /// Set looking state
    pub fn set_looking(&mut self, looking: bool) {
        self.is_looking = looking;
    }
    
    /// Update camera position based on input
    pub fn update(&mut self, dt: f32) -> bool {
        let mut changed = false;
        
        // Calculate forward and right vectors
        let forward = (self.camera.target - self.camera.position).normalize();
        let right = forward.cross(self.camera.up).normalize();
        
        // Calculate movement direction
        let mut movement = Vector3::new(0.0, 0.0, 0.0);
        
        if self.forward {
            movement += forward;
            changed = true;
        }
        if self.backward {
            movement -= forward;
            changed = true;
        }
        if self.right {
            movement += right;
            changed = true;
        }
        if self.left {
            movement -= right;
            changed = true;
        }
        if self.up {
            movement += self.camera.up;
            changed = true;
        }
        if self.down {
            movement -= self.camera.up;
            changed = true;
        }
        
        // Normalize movement vector if not zero
        if movement.magnitude() > 0.0 {
            movement = movement.normalize();
        }
        
        // Apply movement
        let move_amount = self.speed * dt;
        self.camera.position += movement * move_amount;
        self.camera.target += movement * move_amount;
        
        changed
    }
    
    /// Update camera vectors based on yaw and pitch
    fn update_camera_vectors(&mut self) {
        // Calculte new target direction
        let cos_pitch = self.pitch.cos();
        let direction = Vector3::new(
            self.yaw.sin() * cos_pitch,
            self.pitch.sin(),
            self.yaw.cos() * cos_pitch,
        ).normalize();
        
        // Update camera target
        self.camera.target = self.camera.position + direction;
    }
    
    /// Get a reference to the camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    
    /// Get a mutable reference to the camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

/// Conversion matrix between OpenGL and WGPU coordinate systems
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

/// Create a camera buffer 
pub fn create_camera_buffer(
    device: &Device,
    camera: &Camera,
) -> (Buffer, CameraUniform) {
    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(camera);
    
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    
    (camera_buffer, camera_uniform)
}
