#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[rustfmt::skip]
pub const SQUARE_VERTICES: &[Vertex] = &[
     Vertex {position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0]},
     Vertex {position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0]},
     Vertex {position: [-0.5, 0.5, -0.5], tex_coords: [1.0, 0.0]},
     Vertex {position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 1.0]},
     Vertex {position: [0.5, -0.5, -0.5], tex_coords: [0.0, 0.0]},
     Vertex {position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0]},
     Vertex {position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0]},
     Vertex {position: [0.5, 0.5, 0.5], tex_coords: [1.0, 1.0]},
];

#[rustfmt::skip]
pub const SQUARE_INDICES: &[u16] = &[
    0, 6, 2,
    0, 4, 6,
    2, 7, 6,
    2, 3, 7,
    0, 3, 2,
    0, 1, 3,
    1, 7, 3,
    1, 5, 7,
    0, 4, 1,
    1, 4, 5,
    4, 6, 5,
    5, 6, 7,
];

pub struct RenderInstance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl RenderInstance {
    pub fn to_raw(&self) -> RenderInstanceRaw {
        RenderInstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInstanceRaw {
    model: [[f32; 4]; 4],
}

impl RenderInstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RenderInstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
