use crate::core::math::{Vector3, Transform};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vector3,
    pub target:   Vector3,
    pub fov:      f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 2.0, 5.0),
            target:   Vector3::ZERO,
            fov:      60.0_f32.to_radians(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
    pub id:        u64,
    pub transform: Transform,
    pub mesh_id:   Option<u64>,
}

type CpuRenderer = crate::core::renderer_gpu::GpuRenderer<crate::core::present::CpuBufferSink>;

enum RendererVariant {
    None,
    Cpu(CpuRenderer),
    Iron {
        renderer: crate::core::renderer_gpu::GpuRenderer<crate::core::present::CpuBufferSink>,
        iron:     crate::core::present::IrondashTexturePresenter,
    },
}

impl std::fmt::Debug for RendererVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Cpu(_) => write!(f, "Cpu(GpuRenderer)"),
            Self::Iron { .. } => write!(f, "Iron(GpuRenderer+Irondash)"),
        }
    }
}

#[derive(Debug)]
#[flutter_rust_bridge::frb(opaque)]
pub struct Scene3D {
    pub nodes:       Vec<Node>,
    pub camera:      Camera,
    pub light_count: u32,
    pub elapsed:     f32,
    next_id:         u64,
    renderer:        RendererVariant,
    gpu_width:       u32,
    gpu_height:      u32,
    texture_id:      Option<i64>,
}

impl Scene3D {
    pub fn new() -> Self {
        Self {
            nodes:       Vec::new(),
            camera:      Camera::default(),
            light_count: 0,
            elapsed:     0.0,
            next_id:     1,
            renderer:    RendererVariant::None,
            gpu_width:    0,
            gpu_height:   0,
            texture_id:   None,
        }
    }

    pub fn add_node(&mut self, transform: Transform, mesh_id: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(Node { id, transform, mesh_id });
        id
    }

    pub fn update_node_transform(
        &mut self,
        node_id: u64,
        px: f32,
        py: f32,
        pz: f32,
        rx: f32,
        ry: f32,
        rz: f32,
        sx: f32,
        sy: f32,
        sz: f32,
    ) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.transform.position = Vector3::new(px, py, pz);
            node.transform.rotation = Vector3::new(rx, ry, rz);
            node.transform.scale = Vector3::new(sx, sy, sz);
        }
    }

    pub fn update_camera(
        &mut self,
        px: f32,
        py: f32,
        pz: f32,
        tx: f32,
        ty: f32,
        tz: f32,
    ) {
        self.camera.position = Vector3::new(px, py, pz);
        self.camera.target = Vector3::new(tx, ty, tz);
    }

    pub fn get_node(&self, id: u64) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: u64) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn update_elapsed(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    pub fn init_native_texture(&mut self, engine_handle: i64, width: u32, height: u32) -> i64 {
        println!("[scene] Initializing native irondash texture: {}x{}", width, height);
        let iron = crate::core::present::IrondashTexturePresenter::new(engine_handle, width, height);
        let id = iron.texture_id();
        let cpu_sink = crate::core::present::CpuBufferSink::new(width, height);
        let renderer = crate::core::renderer_gpu::GpuRenderer::new(width, height, cpu_sink);
        self.renderer = RendererVariant::Iron { renderer, iron };
        self.gpu_width = width;
        self.gpu_height = height;
        self.texture_id = Some(id);
        id
    }

    pub fn texture_id(&self) -> Option<i64> {
        self.texture_id
    }

    pub fn render_gpu(&mut self, width: u32, height: u32) -> Vec<u8> {
        let need_new = match &self.renderer {
            RendererVariant::None => true,
            _ => self.gpu_width != width || self.gpu_height != height,
        };

        if need_new && self.texture_id.is_none() {
            println!("[scene] Creating/resizing CpuRenderer: {}x{}", width, height);
            let sink = crate::core::present::CpuBufferSink::new(width, height);
            self.renderer = RendererVariant::Cpu(
                crate::core::renderer_gpu::GpuRenderer::new(width, height, sink)
            );
            self.gpu_width = width;
            self.gpu_height = height;
        }

        let (view_proj, eye) =
            crate::core::renderer_gpu::build_view_projection_for_scene(self, width, height);
        let node_transforms: Vec<Transform> =
            self.nodes.iter().map(|n| n.transform).collect();

        match &mut self.renderer {
            RendererVariant::Cpu(r) => {
                r.render_frame(&view_proj, &eye, &node_transforms, width, height)
            }
            RendererVariant::Iron { renderer, iron } => {
                let pixels =
                    renderer.render_frame(&view_proj, &eye, &node_transforms, width, height);
                iron.provider().update_frame(&pixels);
                iron.sendable().mark_frame_available();
                pixels
            }
            RendererVariant::None => vec![0; (width * height * 4) as usize],
        }
    }
}

impl Default for Scene3D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_creation_is_empty() {
        let scene = Scene3D::new();
        assert!(scene.nodes.is_empty());
        assert_eq!(scene.elapsed, 0.0);
        assert_eq!(scene.light_count, 0);
    }

    #[test]
    fn add_custom_node() {
        let mut scene = Scene3D::new();
        let transform = Transform {
            position: Vector3::new(10.0, 20.0, 30.0),
            rotation: Vector3::new(0.1, 0.2, 0.3),
            scale:    Vector3::new(2.0, 2.0, 2.0),
        };
        let id = scene.add_node(transform, Some(42));
        let node = scene.get_node(id).unwrap();
        assert_eq!(node.transform.position, Vector3::new(10.0, 20.0, 30.0));
        assert_eq!(node.transform.rotation.z, 0.3);
        assert_eq!(node.mesh_id, Some(42));
    }

    #[test]
    fn update_node_transform() {
        let mut scene = Scene3D::new();
        let id = scene.add_node(
            Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::ZERO,
                scale: Vector3::ONE,
            },
            Some(100),
        );
        scene.update_node_transform(id, 1.0, 2.0, 3.0, 0.1, 0.2, 0.3, 2.0, 2.0, 2.0);
        let node = scene.get_node(id).unwrap();
        assert_eq!(node.transform.position, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(node.transform.rotation, Vector3::new(0.1, 0.2, 0.3));
        assert_eq!(node.transform.scale, Vector3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn update_node_transform_nonexistent() {
        let mut scene = Scene3D::new();
        scene.update_node_transform(999, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(scene.get_node(999).is_none());
    }

    #[test]
    fn update_camera() {
        let mut scene = Scene3D::new();
        scene.update_camera(10.0, 10.0, 10.0, 0.0, 0.0, -1.0);
        assert_eq!(scene.camera.position, Vector3::new(10.0, 10.0, 10.0));
        assert_eq!(scene.camera.target, Vector3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn update_elapsed() {
        let mut scene = Scene3D::new();
        scene.update_elapsed(0.1);
        scene.update_elapsed(0.2);
        assert!((scene.elapsed - 0.3).abs() < 1e-5);
    }

    #[test]
    fn get_nonexistent_node() {
        let scene = Scene3D::new();
        assert!(scene.get_node(999).is_none());
    }

    #[test]
    fn camera_defaults() {
        let scene = Scene3D::new();
        let cam = scene.camera;
        assert_eq!(cam.position, Vector3::new(0.0, 2.0, 5.0));
        assert_eq!(cam.target, Vector3::ZERO);
    }
}
