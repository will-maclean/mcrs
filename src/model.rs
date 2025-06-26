use crate::texture;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

struct CubeModel {
    // All the faces have indexing (0, 1, 2) and (1, 2, 3)
    pub vertices: [[ModelVertex; 4]; 6],
}

impl Default for CubeModel {
    fn default() -> Self {
        let vertices = [
            // XPos
            [
                ModelVertex {
                    position: [1.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 1.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 0.0, 1.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
            ],
            // XNeg
            [
                ModelVertex {
                    position: [0.0, 0.0, 1.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 1.0, 0.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
            ],
            // YPos
            [
                ModelVertex {
                    position: [0.0, 1.0, 0.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 1.0, 0.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
            ],
            // YNeg
            [
                ModelVertex {
                    position: [0.0, 0.0, 1.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 0.0, 0.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 0.0, 1.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
            ],
            // ZPos
            [
                ModelVertex {
                    position: [1.0, 0.0, 1.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 0.0, 1.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
            ],
            // ZNeg
            [
                ModelVertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [0.0, 1.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 1.0, 0.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                },
                ModelVertex {
                    position: [1.0, 0.0, 0.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 0.0, 0.0],
                },
            ],
        ];

        Self { vertices }
    }
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex_coords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub n_elements: u32,
}

pub struct RenderInstance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: f32,
    pub label: String,
}

impl RenderInstance {
    pub fn to_raw(&self, texture_manger: &texture::TextureManager) -> RenderInstanceRaw {
        RenderInstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation)
                * cgmath::Matrix4::from_scale(self.scale))
            .into(),
            tex_idx: texture_manger.lookup_idx(&self.label).unwrap() as u32,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInstanceRaw {
    model: [[f32; 4]; 4],
    tex_idx: u32,
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
                // texture idx
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
