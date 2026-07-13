use wgpu::util::DeviceExt;

use crate::core::math::Transform;
use crate::core::present::FrameSink;
use crate::core::scene::Scene3D;

const SHADER_SOURCE: &str = include_str!("../shader.wgsl");
const MAX_INSTANCES: u32 = 500;
const MAX_GRID_VERTS: u32 = 2500;
const MAX_GIZMO_VERTS: u32 = 256;
const INSTANCE_DATA_SIZE: u64 = 80;
const GRID_VERTEX_SIZE: u64 = 24;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    model: [[f32; 4]; 4],
    color: [f32; 3],
    _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

const CUBE_VERTICES: [Vertex; 36] = [
    Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5,  0.5, 0.5], normal: [0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: [ 0.5, -0.5, 0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, 0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, 0.5], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5,  0.5, 0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5,  0.5, 0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0] },
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GridVertex {
    position: [f32; 3],
    color: [f32; 3],
}

fn build_grid_vertices(center_x: f32, center_z: f32) -> Vec<GridVertex> {
    let cx = center_x.round() as i32;
    let cz = center_z.round() as i32;
    let color = [0.3, 0.3, 0.35];
    let r = 15i32;
    let mut verts = Vec::new();
    for i in (cx - r)..=(cx + r) {
        verts.push(GridVertex { position: [i as f32, -1.0, (cz - r) as f32], color });
        verts.push(GridVertex { position: [i as f32, -1.0, (cz + r) as f32], color });
    }
    for i in (cz - r)..=(cz + r) {
        verts.push(GridVertex { position: [(cx - r) as f32, -1.0, i as f32], color });
        verts.push(GridVertex { position: [(cx + r) as f32, -1.0, i as f32], color });
    }
    verts
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
}
unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}

#[derive(Debug)]
pub struct GpuRenderer<S: FrameSink = crate::core::present::CpuBufferSink> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    models_storage_buffer: wgpu::Buffer,
    models_bind_group: wgpu::BindGroup,
    models_bind_group_layout: wgpu::BindGroupLayout,
    grid_pipeline: wgpu::RenderPipeline,
    grid_buffer: wgpu::Buffer,
    grid_buffer_capacity: u32,
    gizmo_buffer: wgpu::Buffer,
    gizmo_buffer_capacity: u32,
    render_texture: wgpu::Texture,
    render_texture_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    sink: S,
}

impl<S: FrameSink> GpuRenderer<S> {
    pub fn new(width: u32, height: u32, sink: S) -> Self {
        pollster::block_on(Self::new_async(width, height, sink))
    }

    async fn new_async(width: u32, height: u32, sink: S) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find GPU adapter");

        let info = adapter.get_info();
        println!("[renderer_gpu] Using GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .expect("Failed to create device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let camera_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(80),
                    },
                    count: None,
                }],
            });

        let models_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("models bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new((MAX_INSTANCES as u64) * INSTANCE_DATA_SIZE),
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[Some(&camera_bg_layout), Some(&models_bg_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x3
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let grid_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grid pipeline layout"),
            bind_group_layouts: &[Some(&camera_bg_layout)],
            immediate_size: 0,
        });

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid pipeline"),
            layout: Some(&grid_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("grid_vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GridVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x3
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("grid_fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube vertices"),
            contents: bytemuck::cast_slice(&CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = CUBE_VERTICES.len() as u32;

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bg"),
            layout: &camera_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let models_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("models storage"),
            size: (MAX_INSTANCES as u64) * INSTANCE_DATA_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let models_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("models bg"),
            layout: &models_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: models_storage_buffer.as_entire_binding(),
            }],
        });

        let grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid staging"),
            size: (MAX_GRID_VERTS as u64) * GRID_VERTEX_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let gizmo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gizmo staging"),
            size: (MAX_GIZMO_VERTS as u64) * GRID_VERTEX_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let render_texture_view = render_texture.create_view(&Default::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&Default::default());

        Self {
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            camera_buffer,
            camera_bind_group,
            models_storage_buffer,
            models_bind_group,
            models_bind_group_layout: models_bg_layout,
            grid_pipeline,
            grid_buffer,
            grid_buffer_capacity: MAX_GRID_VERTS,
            gizmo_buffer,
            gizmo_buffer_capacity: MAX_GIZMO_VERTS,
            render_texture,
            render_texture_view,
            depth_texture,
            depth_view,
            sink,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.render_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.render_texture_view = self.render_texture.create_view(&Default::default());

        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_view = self.depth_texture.create_view(&Default::default());
    }

    pub fn render_frame(
        &mut self,
        view_proj: &glam::Mat4,
        eye: &glam::Vec3,
        model_matrices: &[[[f32; 4]; 4]],
        colors: &[[f32; 3]],
        gizmo_lines: &[([f32; 3], [f32; 3])],
        gizmo_colors: &[[f32; 3]; 3],
        width: u32,
        height: u32,
        player_x: f32,
        player_z: f32,
    ) -> Vec<u8> {
        if self.render_texture.width() != width || self.render_texture.height() != height {
            self.resize(width, height);
        }

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: view_proj.to_cols_array_2d(),
                camera_pos: [eye.x, eye.y, eye.z, 1.0],
            }),
        );

        let num_instances = model_matrices.len().min(MAX_INSTANCES as usize) as u32;

        if num_instances > 0 {
            let instances: Vec<InstanceData> = model_matrices.iter()
                .zip(colors.iter())
                .map(|(m, c)| InstanceData { model: *m, color: *c, _pad: 0.0 })
                .collect();
            self.queue.write_buffer(
                &self.models_storage_buffer,
                0,
                bytemuck::cast_slice(&instances),
            );
        }

        let gizmo_vert_count = (gizmo_lines.len() * 2) as u32;

        if gizmo_vert_count > 0 {
            let verts: Vec<GridVertex> = gizmo_lines
                .iter()
                .enumerate()
                .flat_map(|(i, (a, b))| {
                    let col = gizmo_colors[i % 3];
                    vec![
                        GridVertex { position: *a, color: col },
                        GridVertex { position: *b, color: col },
                    ]
                })
                .collect();
            if verts.len() <= self.gizmo_buffer_capacity as usize {
                self.queue.write_buffer(&self.gizmo_buffer, 0, bytemuck::cast_slice(&verts));
            }
        }

        let grid_verts = build_grid_vertices(player_x, player_z);
        let grid_vcount = grid_verts.len().min(self.grid_buffer_capacity as usize) as u32;
        self.queue.write_buffer(&self.grid_buffer, 0, bytemuck::cast_slice(&grid_verts[..grid_vcount as usize]));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.12, g: 0.12, b: 0.16, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_pipeline(&self.grid_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.grid_buffer.slice(..));
            rpass.draw(0..grid_vcount, 0..1);

            if gizmo_vert_count > 0 {
                rpass.set_vertex_buffer(0, self.gizmo_buffer.slice(..));
                rpass.draw(0..gizmo_vert_count, 0..1);
            }

            if num_instances > 0 {
                rpass.set_pipeline(&self.render_pipeline);
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);
                rpass.set_bind_group(1, &self.models_bind_group, &[]);
                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rpass.draw(0..self.num_vertices, 0..num_instances);
            }
        }
        self.queue.submit(Some(encoder.finish()));

        self.sink
            .present(&self.device, &self.queue, &self.render_texture)
    }
}

pub fn build_view_projection_for_scene(
    scene: &Scene3D,
    width: u32,
    height: u32,
) -> (glam::Mat4, glam::Vec3) {
    let eye = glam::Vec3::new(
        scene.camera.position.x,
        scene.camera.position.y,
        scene.camera.position.z,
    );
    let target = glam::Vec3::new(
        scene.camera.target.x,
        scene.camera.target.y,
        scene.camera.target.z,
    );
    let up = glam::Vec3::Y;
    let view = glam::Mat4::look_at_rh(eye, target, up);
    let aspect = width as f32 / height as f32;
    let proj = glam::Mat4::perspective_rh(scene.camera.fov, aspect, 0.1, 100.0);
    (proj * view, eye)
}

pub fn build_view_proj_matrix(scene: &crate::core::scene::Scene3D, width: u32, height: u32) -> glam::Mat4 {
    build_view_projection_for_scene(scene, width, height).0
}

pub fn build_model_matrix(t: &Transform) -> glam::Mat4 {
    let translation = glam::Vec3::new(t.position.x, t.position.y, t.position.z);
    let scale = glam::Vec3::new(t.scale.x, t.scale.y, t.scale.z);
    let rotation = glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        t.rotation.x,
        t.rotation.y,
        t.rotation.z,
    );
    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

pub fn build_model_matrices(transforms: &[Transform]) -> Vec<[[f32; 4]; 4]> {
    transforms
        .iter()
        .map(|t| build_model_matrix(t).to_cols_array_2d())
        .collect()
}
