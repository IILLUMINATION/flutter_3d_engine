use std::sync::atomic::{AtomicU64, Ordering};

use wgpu::util::DeviceExt;

use crate::core::math::Transform;
use crate::core::present::FrameSink;
use crate::core::scene::Scene3D;

const SHADER_SOURCE: &str = include_str!("../shader.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

const CUBE_VERTICES: [Vertex; 36] = [
    Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0] },

    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0] },

    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 1.0,  0.0,  0.0] },

    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0,  0.0,  0.0] },

    Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0] },

    Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0] },
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GridVertex {
    position: [f32; 3],
}

fn build_grid_vertices() -> Vec<GridVertex> {
    let mut verts = Vec::new();
    for i in -5..=5 {
        let f = i as f32;
        verts.push(GridVertex { position: [f, -1.0, -5.0] });
        verts.push(GridVertex { position: [f, -1.0,  5.0] });
        verts.push(GridVertex { position: [-5.0, -1.0, f] });
        verts.push(GridVertex { position: [ 5.0, -1.0, f] });
    }
    verts
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct GpuRenderer<S: FrameSink = crate::core::present::CpuBufferSink> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    grid_pipeline: wgpu::RenderPipeline,
    grid_vertex_buffer: wgpu::Buffer,
    grid_num_vertices: u32,
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

        let model_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("model bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(64),
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[Some(&camera_bg_layout), Some(&model_bg_layout)],
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
                        0 => Float32x3,  // position
                        1 => Float32x3   // normal
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

        // --- grid pipeline: no model BGL, uses camera BGL only, LineList ---
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
                        0 => Float32x3
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

        let grid_vertices = build_grid_vertices();
        let grid_num_vertices = grid_vertices.len() as u32;
        let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid vertices"),
            contents: bytemuck::cast_slice(&grid_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

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

        println!("[renderer_gpu] Initialized {}x{}", width, height);

        Self {
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            camera_buffer,
            camera_bind_group,
            model_bind_group_layout: model_bg_layout,
            grid_pipeline,
            grid_vertex_buffer,
            grid_num_vertices,
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
        node_transforms: &[Transform],
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        if n % 60 == 0 {
            println!("[renderer_gpu] render_frame call #{}", n + 1);
        }

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

            // draw ground grid
            rpass.set_pipeline(&self.grid_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
            rpass.draw(0..self.grid_num_vertices, 0..1);

            // draw cubes
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            for transform in node_transforms {
                let model = build_model_matrix(transform);
                let model_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("model uniform"),
                    contents: bytemuck::bytes_of(&ModelUniform {
                        model: model.to_cols_array_2d(),
                    }),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                let model_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("model bg"),
                    layout: &self.model_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buf.as_entire_binding(),
                    }],
                });
                rpass.set_bind_group(1, &model_bg, &[]);
                rpass.draw(0..self.num_vertices, 0..1);
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

fn build_model_matrix(t: &Transform) -> glam::Mat4 {
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
