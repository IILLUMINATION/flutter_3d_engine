use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::math::Transform;
use crate::core::present::FrameSink;

const SHADER_SOURCE: &str = include_str!("../shader.wgsl");

#[derive(Debug)]
pub struct GpuRenderer<S: FrameSink = crate::core::present::CpuBufferSink> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    render_texture: wgpu::Texture,
    render_texture_view: wgpu::TextureView,
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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let render_texture_view = render_texture.create_view(&Default::default());

        println!("[renderer_gpu] Initialized {}x{}", width, height);

        Self {
            device,
            queue,
            render_pipeline,
            render_texture,
            render_texture_view,
            sink,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.render_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.render_texture_view = self.render_texture.create_view(&Default::default());
    }

    pub fn render_frame(
        &mut self,
        _view_proj: &[[f32; 4]; 4],
        _node_transforms: &[Transform],
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
                            r: 0.12,
                            g: 0.12,
                            b: 0.16,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));

        self.sink
            .present(&self.device, &self.queue, &self.render_texture)
    }
}

pub fn build_view_projection_for_scene(
    scene: &crate::core::scene::Scene3D,
    width: u32,
    height: u32,
) -> [[f32; 4]; 4] {
    let cam = &scene.camera;
    let aspect = width as f32 / height as f32;
    let proj = build_perspective_matrix(cam.fov, aspect, 0.1, 100.0);
    let view = build_view_matrix(&cam.position, &cam.target, &crate::core::math::Vector3::UP);
    multiply_mat4(&proj, &view)
}

fn build_perspective_matrix(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov / 2.0).tan();
    let range_inv = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) * range_inv, far * near * range_inv * 2.0],
        [0.0, 0.0, -1.0, 0.0],
    ]
}

fn build_view_matrix(
    eye: &crate::core::math::Vector3,
    target: &crate::core::math::Vector3,
    up: &crate::core::math::Vector3,
) -> [[f32; 4]; 4] {
    let f = (*target - *eye).normalize_or_zero();
    let s = {
        let cr = crate::core::math::Vector3::new(
            f.y * up.z - f.z * up.y,
            f.z * up.x - f.x * up.z,
            f.x * up.y - f.y * up.x,
        );
        cr.normalize_or_zero()
    };
    let u = crate::core::math::Vector3::new(
        s.y * f.z - s.z * f.y,
        s.z * f.x - s.x * f.z,
        s.x * f.y - s.y * f.x,
    );
    [
        [s.x, u.x, -f.x, 0.0],
        [s.y, u.y, -f.y, 0.0],
        [s.z, u.z, -f.z, 0.0],
        [-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
    ]
}

fn multiply_mat4(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            out[row][col] = a[row][0] * b[0][col]
                + a[row][1] * b[1][col]
                + a[row][2] * b[2][col]
                + a[row][3] * b[3][col];
        }
    }
    out
}
