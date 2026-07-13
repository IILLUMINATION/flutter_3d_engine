struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
}

struct InstanceData {
    model: mat4x4<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<storage, read> instances: array<InstanceData>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_idx: u32) -> VertexOutput {
    let inst = instances[instance_idx];
    let world_pos = inst.model * vec4<f32>(in.position, 1.0);
    let clip_pos = camera.view_proj * world_pos;
    var out: VertexOutput;
    out.clip_position = clip_pos;
    out.world_pos = world_pos.xyz;
    out.normal = (inst.model * vec4<f32>(in.normal, 0.0)).xyz;
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);

    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.25));
    let diffuse = max(dot(n, light_dir), 0.0);

    let view_dir = normalize(camera.camera_pos.xyz - in.world_pos);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(n, half_dir), 0.0), 32.0) * 0.5;

    let fill_dir = normalize(vec3<f32>(-0.5, -0.5, -0.25));
    let fill_diffuse = max(dot(n, fill_dir), 0.0) * 0.2;

    let ambient = 0.08;
    let lit = (ambient + diffuse + fill_diffuse) * in.color + vec3<f32>(specular);

    return vec4<f32>(lit, 1.0);
}

// --- grid / gizmo line shader ---

struct GridVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct GridVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn grid_vs_main(in: GridVertexInput) -> GridVertexOutput {
    var out: GridVertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn grid_fs_main(in: GridVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
