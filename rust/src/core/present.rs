pub trait FrameSink: Send + Sync {
    fn present(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &wgpu::Texture,
    ) -> Vec<u8>;
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

    fn ensure_buffer(&mut self, device: &wgpu::Device) {
        if self.output_buffer.is_none() {
            self.output_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cpu sink buffer"),
                size: self.output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
        }
    }
}

impl FrameSink for CpuBufferSink {
    fn present(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &wgpu::Texture,
    ) -> Vec<u8> {
        self.ensure_buffer(device);
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

        rx.recv()
            .unwrap()
            .expect("Failed to map output buffer in CpuBufferSink");

        let data = buffer_slice
            .get_mapped_range()
            .expect("Failed to get mapped range");
        let unpadded = self.width as usize * 4;
        let mut out =
            Vec::with_capacity((self.width as usize) * (self.height as usize) * 4);

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
