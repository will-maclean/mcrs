fn face_rotation_matrix(face_idx: u32) -> mat3x3<f32> {
    if face_idx == 0u { // +Z
        return mat3x3<f32>(
            vec3<f32>(1.0, 0.0, 0.0),
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(0.0, 0.0, 1.0),
        );
    } else if face_idx == 1u { // -Z
        return mat3x3<f32>(
            vec3<f32>(-1.0, 0.0,  0.0),
            vec3<f32>( 0.0, 1.0,  0.0),
            vec3<f32>( 0.0, 0.0, -1.0),
        );
    } else if face_idx == 2u { // -X
        return mat3x3<f32>(
            vec3<f32>(0.0, 0.0, -1.0),
            vec3<f32>(0.0, 1.0,  0.0),
            vec3<f32>(-1.0, 0.0, 0.0),
        );
    } else if face_idx == 3u { // +X
        return mat3x3<f32>(
            vec3<f32>(0.0, 0.0, 1.0),
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(1.0, 0.0, 0.0),
        );
    } else if face_idx == 4u { // +Y
        return mat3x3<f32>(
            vec3<f32>(1.0, 0.0, 0.0),
            vec3<f32>(0.0, 0.0, 1.0),
            vec3<f32>(0.0, 1.0, 0.0),
        );
    } else { // 5: -Y
        return mat3x3<f32>(
            vec3<f32>(1.0,  0.0,  0.0),
            vec3<f32>(0.0,  0.0, -1.0),
            vec3<f32>(0.0, -1.0,  0.0),
        );
    }
}

// Vertex shader
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) tex_idx: u32,
    @location(10) face_idx: u32,
}
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_idx: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let face_rotation = face_rotation_matrix(instance.face_idx);
    let rotated_position = face_rotation * model.position;

    let world_position = model_matrix * vec4<f32>(rotated_position, 1.0);
    let clip_position = camera.view_proj * world_position;

    var out: VertexOutput;
    out.clip_position = clip_position;
    out.tex_coords = model.tex_coords;
    out.tex_idx = instance.tex_idx;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var texture_array: texture_2d_array<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_array, tex_sampler, in.tex_coords, in.tex_idx);
}

