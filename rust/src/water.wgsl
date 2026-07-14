struct WaterVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) ao: f32,
}

struct WaterVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) ao: f32,
}

struct WaterUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    time: f32,
    screen_size: vec2<f32>,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> water_uniforms: WaterUniforms;
@group(0) @binding(1) var normal_sampler: sampler;
@group(0) @binding(2) var normal_map: texture_2d<f32>;
@group(0) @binding(3) var opaque_sampler: sampler;
@group(0) @binding(4) var opaque_texture: texture_2d<f32>;

@vertex
fn water_vs(in: WaterVertexInput) -> WaterVertexOutput {
    var out: WaterVertexOutput;
    out.world_pos = in.position;
    out.position = water_uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.ao = in.ao;
    return out;
}

@fragment
fn water_fs(in: WaterVertexOutput) -> @location(0) vec4<f32> {
    let n1 = textureSample(normal_map, normal_sampler, in.uv * 2.0 + water_uniforms.time * 0.15).rgb * 2.0 - 1.0;
    let n2 = textureSample(normal_map, normal_sampler, in.uv * 3.0 - water_uniforms.time * 0.2).rgb * 2.0 - 1.0;
    let normal = normalize(n1 + n2 + vec3<f32>(0.0, 1.0, 0.0));

    let clip_xy = in.position.xy / in.position.w;
    let screen_uv = (clip_xy + 1.0) * 0.5;
    let screen_uv_adj = vec2<f32>(screen_uv.x, 1.0 - screen_uv.y);

    let distort = normal.xz * 0.04;
    let bg = textureSample(opaque_texture, opaque_sampler, screen_uv_adj + distort).rgb;

    let view_dir = normalize(water_uniforms.camera_pos.xyz - in.world_pos);
    let fresnel = pow(1.0 - max(dot(view_dir, normal), 0.0), 3.0);

    let water_color = vec3<f32>(0.15, 0.45, 0.75);
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.25));
    let spec = pow(max(dot(reflect(-light_dir, normal), view_dir), 0.0), 64.0) * 0.6;
    let spec_color = vec3<f32>(1.0, 0.95, 0.8);

    let refraction = bg * water_color;
    let reflection = mix(water_color * 1.3, vec3<f32>(0.8, 0.9, 1.0), 0.5);
    let color = mix(refraction, reflection, fresnel) + spec_color * spec;

    let depth_fade = min(water_uniforms.time * 0.01, 1.0);
    let aoed = color * (0.6 + in.ao * 0.4);

    return vec4<f32>(aoed, 0.75 + fresnel * 0.25);
}
