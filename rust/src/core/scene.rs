use rapier3d::prelude::*;
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
    pub rb_handle: Option<RigidBodyHandle>,
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

    // Rapier physics
    gravity:             Vector3,
    rigid_body_set:      RigidBodySet,
    collider_set:        ColliderSet,
    query_pipeline:      QueryPipeline,
    integration_parameters: IntegrationParameters,
    physics_pipeline:    PhysicsPipeline,
    island_manager:      IslandManager,
    broad_phase:         BroadPhaseMultiSap,
    narrow_phase:        NarrowPhase,
    impulse_joint_set:   ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver:          CCDSolver,

    // ray-picking
    pub dragged_body: Option<(RigidBodyHandle, f32)>,
}

impl Scene3D {
    pub fn new() -> Self {
        let gravity = Vector3::new(0.0, -9.81, 0.0);

        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // --- ground: static rigid body at Y = -1.0 ---
        let ground_rb = RigidBodyBuilder::fixed()
            .translation(vector![0.0, -1.0, 0.0])
            .build();
        let ground_handle = rigid_body_set.insert(ground_rb);
        let ground_collider = ColliderBuilder::cuboid(10.0, 0.1, 10.0).build();
        collider_set.insert_with_parent(ground_collider, ground_handle, &mut rigid_body_set);

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
            gravity,
            rigid_body_set,
            collider_set,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline:       PhysicsPipeline::new(),
            island_manager:         IslandManager::new(),
            broad_phase:            BroadPhaseMultiSap::new(),
            narrow_phase:           NarrowPhase::new(),
            impulse_joint_set:      ImpulseJointSet::new(),
            multibody_joint_set:    MultibodyJointSet::new(),
            ccd_solver:             CCDSolver::new(),
            query_pipeline:         QueryPipeline::new(),
            dragged_body:           None,
        }
    }

    pub fn add_cube_physics(&mut self, px: f32, py: f32, pz: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![px, py, pz])
            .linear_damping(0.2)
            .angular_damping(1.0)
            .build();
        let rb_handle = self.rigid_body_set.insert(rb);
        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5)
            .restitution(0.5)
            .build();
        self.collider_set.insert_with_parent(collider, rb_handle, &mut self.rigid_body_set);

        self.nodes.push(Node {
            id,
            transform: Transform {
                position: Vector3::new(px, py, pz),
                rotation: Vector3::ZERO,
                scale:    Vector3::ONE,
            },
            mesh_id:   Some(100u64),
            rb_handle: Some(rb_handle),
        });
        id
    }

    pub fn physics_step(&mut self, _dt: f32) {
        self.physics_pipeline.step(
            &vector![self.gravity.x, self.gravity.y, self.gravity.z],
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );

        // sync physics transforms back to nodes
        for node in &mut self.nodes {
            if let Some(handle) = node.rb_handle {
                if let Some(rb) = self.rigid_body_set.get(handle) {
                    let pos = rb.translation();
                    node.transform.position = Vector3::new(pos.x, pos.y, pos.z);
                    let rot = rb.rotation();
                    let (roll, pitch, yaw) = rot.euler_angles();
                    node.transform.rotation = Vector3::new(roll, pitch, yaw);
                }
            }
        }
    }

    pub fn update_node_transform(
        &mut self,
        node_id: u64,
        px: f32, py: f32, pz: f32,
        rx: f32, ry: f32, rz: f32,
        sx: f32, sy: f32, sz: f32,
    ) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.transform.position = Vector3::new(px, py, pz);
            node.transform.rotation = Vector3::new(rx, ry, rz);
            node.transform.scale = Vector3::new(sx, sy, sz);
        }
    }

    pub fn update_camera(
        &mut self,
        px: f32, py: f32, pz: f32,
        tx: f32, ty: f32, tz: f32,
    ) {
        self.camera.position = Vector3::new(px, py, pz);
        self.camera.target = Vector3::new(tx, ty, tz);
    }

    pub fn get_node(&self, id: u64) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
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

    fn build_ray(&self, screen_x: f32, screen_y: f32, screen_width: f32, screen_height: f32) -> (glam::Vec3, glam::Vec3) {
        let view_proj = crate::core::renderer_gpu::build_view_proj_matrix(self, screen_width as u32, screen_height as u32);
        let inv_vp = view_proj.inverse();

        let ndc_x = (screen_x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / screen_height) * 2.0;

        let near = inv_vp * glam::Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let near = near / near.w;
        let far = inv_vp * glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        let far = far / far.w;

        let origin = glam::Vec3::new(near.x, near.y, near.z);
        let dir = (glam::Vec3::new(far.x, far.y, far.z) - origin).normalize();
        (origin, dir)
    }

    pub fn handle_pointer_down(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> bool {
        let (origin, dir) = self.build_ray(screen_x, screen_y, screen_width, screen_height);
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![dir.x, dir.y, dir.z],
        );

        let max_toi = 100.0;
        let solid = true;
        let filter = QueryFilter::default();

        self.query_pipeline.update(&self.collider_set);

        if let Some((collider_handle, _toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            solid,
            filter,
        ) {
            if let Some(rb_handle) = self.collider_set.get(collider_handle).and_then(|c| c.parent()) {
                let current_y = self.rigid_body_set.get(rb_handle).map_or(0.0, |rb| rb.translation().y);
                self.dragged_body = Some((rb_handle, current_y));
                return true;
            }
        }
        false
    }

    pub fn handle_pointer_move(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        if let Some((handle, target_y)) = self.dragged_body {
            let (origin, dir) = self.build_ray(screen_x, screen_y, screen_width, screen_height);

            if dir.y.abs() > 0.0001 {
                let t = (target_y - origin.y) / dir.y;
                let mut new_pos = origin + dir * t;

                new_pos.x = new_pos.x.clamp(-6.0, 6.0);
                new_pos.z = new_pos.z.clamp(-6.0, 6.0);
                new_pos.y = target_y;

                if let Some(rb) = self.rigid_body_set.get_mut(handle) {
                    rb.set_linvel(vector![0.0, 0.0, 0.0], true);
                    rb.set_angvel(vector![0.0, 0.0, 0.0], true);
                    rb.set_translation(vector![new_pos.x, new_pos.y, new_pos.z], true);
                }
            }
        }
    }

    pub fn handle_pointer_up(&mut self) {
        self.dragged_body = None;
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

impl std::fmt::Debug for Scene3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene3D")
            .field("nodes", &self.nodes.len())
            .field("camera", &self.camera)
            .field("elapsed", &self.elapsed)
            .field("gravity", &self.gravity)
            .finish()
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
    fn add_cube_physics_creates_node_and_rigid_body() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(0.0, 5.0, 0.0);
        let node = scene.get_node(id).unwrap();
        assert_eq!(node.transform.position, Vector3::new(0.0, 5.0, 0.0));
        assert!(node.rb_handle.is_some());
        assert_eq!(node.mesh_id, Some(100));
    }

    #[test]
    fn physics_step_moves_cube_down() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(0.0, 5.0, 0.0);
        let y_before = scene.get_node(id).unwrap().transform.position.y;
        scene.physics_step(0.016);
        let y_after = scene.get_node(id).unwrap().transform.position.y;
        assert!(
            y_after < y_before,
            "cube should fall downward: before={} after={}",
            y_before, y_after
        );
    }

    #[test]
    fn physics_step_cube_doesnt_fall_through_ground() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(0.0, 5.0, 0.0);
        for _ in 0..300 {
            scene.physics_step(0.016);
        }
        let y = scene.get_node(id).unwrap().transform.position.y;
        assert!(
            y >= -0.6,
            "cube should rest on ground (y >= -0.6), got y={}",
            y
        );
    }

    #[test]
    fn add_custom_node_no_physics() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(10.0, 20.0, 30.0);
        let node = scene.get_node(id).unwrap();
        assert_eq!(node.transform.position.x, 10.0);
        assert!(node.rb_handle.is_some());
    }

    #[test]
    fn update_node_transform() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(0.0, 0.0, 0.0);
        scene.update_node_transform(id, 1.0, 2.0, 3.0, 0.1, 0.2, 0.3, 2.0, 2.0, 2.0);
        let node = scene.get_node(id).unwrap();
        assert_eq!(node.transform.position, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(node.transform.rotation, Vector3::new(0.1, 0.2, 0.3));
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
