struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    let clip_pos = camera.view_proj * world_pos;
    var out: VertexOutput;
    out.clip_position = clip_pos;
    out.normal = (model.model * vec4<f32>(in.normal, 0.0)).xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.25));
    let diffuse = max(dot(normalize(in.normal), light_dir), 0.0);
    let base_color = vec3<f32>(0.1, 0.3, 0.7);
    let final_color = base_color * (diffuse + 0.15);
    return vec4<f32>(final_color, 1.0);
}
