use std::sync::{Arc, Mutex};

use irondash_texture::{
    BoxedPixelData, PayloadProvider, SendableTexture, SimplePixelData, Texture,
};

pub trait FrameSink: Send + Sync {
    fn present(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &wgpu::Texture,
    ) -> Vec<u8>;

    fn present_from_dual(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        opaque: &wgpu::Texture,
        composite: &wgpu::Texture,
    ) -> Vec<u8> {
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: opaque,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: composite,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: composite.width(),
                height: composite.height(),
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(encoder.finish()));

        self.present(device, queue, composite)
    }
}

#[derive(Debug)]
pub struct CpuBufferSink {
    output_buffer: Option<wgpu::Buffer>,
    output_buffer_size: u64,
    padded_bytes_per_row: u32,
    width: u32,
    height: u32,
}

impl CpuBufferSink {
    pub fn new(width: u32, height: u32) -> Self {
        let unpadded = width * 4;
        let padded = pad_to_alignment(unpadded, 256);
        Self {
            output_buffer: None,
            output_buffer_size: (padded as u64) * (height as u64),
            padded_bytes_per_row: padded,
            width,
            height,
        }
    }

    fn ensure_buffer(&mut self, device: &wgpu::Device) -> bool {
        if self.output_buffer.is_none() {
            self.output_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cpu sink buffer"),
                size: self.output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
        }
        self.output_buffer.is_some()
    }
}

impl FrameSink for CpuBufferSink {
    fn present(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &wgpu::Texture,
    ) -> Vec<u8> {
        if !self.ensure_buffer(device) {
            return vec![0u8; (self.width as usize) * (self.height as usize) * 4];
        }
        let buffer = self.output_buffer.as_ref().unwrap();

        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: frame,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).ok();
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .ok();

        let Ok(inner) = rx.recv() else {
            return vec![0u8; (self.width as usize) * (self.height as usize) * 4];
        };
        let Ok(()) = inner else {
            return vec![0u8; (self.width as usize) * (self.height as usize) * 4];
        };

        let Ok(data) = buffer_slice.get_mapped_range() else {
            return vec![0u8; (self.width as usize) * (self.height as usize) * 4];
        };
        let unpadded = self.width as usize * 4;
        let mut out = Vec::with_capacity((self.width as usize) * (self.height as usize) * 4);

        for row in 0..(self.height as usize) {
            let start = row * self.padded_bytes_per_row as usize;
            out.extend_from_slice(&data[start..start + unpadded]);
        }

        drop(data);
        buffer.unmap();

        out
    }
}

fn pad_to_alignment(value: u32, alignment: u32) -> u32 {
    ((value + alignment - 1) / alignment) * alignment
}

pub struct IrondashTexturePresenter {
    sendable: Arc<SendableTexture<BoxedPixelData>>,
    provider: Arc<TexturePayloadProvider>,
    texture_id: i64,
}

impl IrondashTexturePresenter {
    pub fn new(engine_handle: i64, width: u32, height: u32) -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<Self>();
        irondash_engine_context::EngineContext::perform_on_main_thread(move || {
            let provider = Arc::new(TexturePayloadProvider::new(width, height));
            let provider_clone: Arc<dyn PayloadProvider<BoxedPixelData>> = provider.clone();
            let texture = Texture::new_with_provider(engine_handle, provider_clone)
                .expect("Failed to create irondash texture");
            let id = texture.id();
            let sendable = texture.into_sendable_texture();
            println!("[irondash] Texture registered, id={} (SendableTexture)", id);
            tx.send(Self {
                sendable,
                provider,
                texture_id: id,
            }).ok();
        })
        .expect("Failed to run on main thread");
        rx.recv().expect("Irondash Texture init failed")
    }

    pub fn texture_id(&self) -> i64 {
        self.texture_id
    }

    pub fn sendable(&self) -> &Arc<SendableTexture<BoxedPixelData>> {
        &self.sendable
    }

    pub fn provider(&self) -> &Arc<TexturePayloadProvider> {
        &self.provider
    }
}

pub struct TexturePayloadProvider {
    width: u32,
    height: u32,
    latest_frame: Mutex<Vec<u8>>,
}

impl TexturePayloadProvider {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            latest_frame: Mutex::new(vec![0u8; (width as usize) * (height as usize) * 4]),
        }
    }

    pub fn update_frame(&self, pixels: &[u8]) {
        let mut frame = self.latest_frame.lock().unwrap();
        frame.copy_from_slice(pixels);
    }
}

impl PayloadProvider<BoxedPixelData> for TexturePayloadProvider {
    fn get_payload(&self) -> BoxedPixelData {
        let frame = self.latest_frame.lock().unwrap();
        SimplePixelData::new_boxed(self.width as i32, self.height as i32, frame.clone())
    }
}
