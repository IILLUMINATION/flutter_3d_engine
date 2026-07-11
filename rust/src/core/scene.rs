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

#[derive(Debug)]
#[flutter_rust_bridge::frb(opaque)]
pub struct Scene3D {
    pub nodes:       Vec<Node>,
    pub camera:      Camera,
    pub light_count: u32,
    pub elapsed:     f32,
    next_id:         u64,
    gpu_renderer:    Option<crate::core::renderer_gpu::GpuRenderer<crate::core::present::CpuBufferSink>>,
    gpu_width:       u32,
    gpu_height:      u32,
}

impl Scene3D {
    pub fn new() -> Self {
        Self {
            nodes:       Vec::new(),
            camera:      Camera::default(),
            light_count: 0,
            elapsed:     0.0,
            next_id:     1,
            gpu_renderer: None,
            gpu_width:    0,
            gpu_height:   0,
        }
    }

    pub fn add_node(&mut self, transform: Transform, mesh_id: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(Node { id, transform, mesh_id });
        id
    }

    pub fn add_test_cube(&mut self) -> u64 {
        let transform = Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::ZERO,
            scale:    Vector3::ONE,
        };
        self.add_node(transform, Some(100u64))
    }

    pub fn get_node(&self, id: u64) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: u64) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;

        let angular_velocity = 1.0; // radians per second around Y axis
        for node in &mut self.nodes {
            node.transform.rotation.y += angular_velocity * dt;
        }
    }

    pub fn render_gpu(&mut self, width: u32, height: u32) -> Vec<u8> {
        if self.gpu_renderer.is_none() || self.gpu_width != width || self.gpu_height != height {
            println!("[scene] Creating/resizing GpuRenderer: {}x{}", width, height);
            let sink = crate::core::present::CpuBufferSink::new(width, height);
            self.gpu_renderer = Some(crate::core::renderer_gpu::GpuRenderer::new(width, height, sink));
            self.gpu_width = width;
            self.gpu_height = height;
        }
        // Scene data needed by renderer — snapshot before mutable borrow
        let view_proj = crate::core::renderer_gpu::build_view_projection_for_scene(self, width, height);
        let node_transforms: Vec<crate::core::math::Transform> = self.nodes.iter().map(|n| n.transform).collect();
        let renderer = self.gpu_renderer.as_mut().unwrap();
        renderer.render_frame(&view_proj, &node_transforms, width, height)
    }
}

impl Default for Scene3D {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn scene_creation_is_empty() {
        let scene = Scene3D::new();
        assert!(scene.nodes.is_empty());
        assert_eq!(scene.elapsed, 0.0);
        assert_eq!(scene.light_count, 0);
    }

    #[test]
    fn add_test_cube() {
        let mut scene = Scene3D::new();
        let id = scene.add_test_cube();

        let cube = scene.get_node(id).expect("cube must exist");
        assert_eq!(cube.transform.position, Vector3::ZERO);
        assert_eq!(cube.transform.rotation, Vector3::ZERO);
        assert_eq!(cube.mesh_id, Some(100));
    }

    #[test]
    fn update_rotates_nodes() {
        let mut scene = Scene3D::new();
        let id = scene.add_test_cube();

        let dt = PI / 2.0; // 90 degrees in radians
        scene.update(dt);

        let cube = scene.get_node(id).unwrap();
        let epsilon = 1e-5;
        assert!((cube.transform.rotation.y - PI / 2.0).abs() < epsilon,
                "Expected ~PI/2 rotation, got {}", cube.transform.rotation.y);
        assert_eq!(cube.transform.rotation.x, 0.0);
        assert_eq!(cube.transform.rotation.z, 0.0);
    }

    #[test]
    fn update_accumulates_rotation_over_multiple_frames() {
        let mut scene = Scene3D::new();
        let id = scene.add_test_cube();

        let dt = 0.5;
        scene.update(dt);
        scene.update(dt);
        scene.update(dt);

        let cube = scene.get_node(id).unwrap();
        let epsilon = 1e-5;
        assert!((cube.transform.rotation.y - 1.5).abs() < epsilon,
                "Expected 1.5 rotation after 3 frames, got {}", cube.transform.rotation.y);
    }

    #[test]
    fn update_increments_elapsed() {
        let mut scene = Scene3D::new();
        scene.update(0.1);
        scene.update(0.2);
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
}
