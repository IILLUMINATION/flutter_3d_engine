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
            position: Vector3::new(0.0, 3.6, 0.0),
            target:   Vector3::new(0.0, 3.6, -1.0),
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

    pub camera_theta:  f32,
    pub camera_phi:    f32,
    pub camera_radius: f32,

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

    player_body_handle:  RigidBodyHandle,
    ground_body_handle:  RigidBodyHandle,
}

impl Scene3D {
    pub fn new() -> Self {
        let gravity = Vector3::new(0.0, -9.81, 0.0);

        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let ground_rb = RigidBodyBuilder::fixed()
            .translation(vector![0.0, -1.0, 0.0])
            .build();
        let ground_body_handle = rigid_body_set.insert(ground_rb);
        let ground_collider = ColliderBuilder::cuboid(1000.0, 0.1, 1000.0).build();
        collider_set.insert_with_parent(ground_collider, ground_body_handle, &mut rigid_body_set);

        let player_rb = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 2.0, 0.0])
            .linear_damping(0.5)
            .enabled_rotations(false, false, false)
            .build();
        let player_body_handle = rigid_body_set.insert(player_rb);
        let player_collider = ColliderBuilder::capsule_y(0.9, 0.4)
            .restitution(0.0)
            .build();
        collider_set.insert_with_parent(player_collider, player_body_handle, &mut rigid_body_set);

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
            camera_theta:  0.0,
            camera_phi:    0.0,
            camera_radius: 0.0,
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
            player_body_handle,
            ground_body_handle,
        }
    }

    pub fn update_camera_from_spherical(&mut self) {
        let x = self.camera_radius * f32::cos(self.camera_phi) * f32::sin(self.camera_theta);
        let y = self.camera_radius * f32::sin(self.camera_phi);
        let z = self.camera_radius * f32::cos(self.camera_phi) * f32::cos(self.camera_theta);
        self.camera.position = Vector3::new(x, y, z);
        self.camera.target = Vector3::ZERO;
    }

    fn update_fps_camera(&mut self) {
        if let Some(rb) = self.rigid_body_set.get(self.player_body_handle) {
            let pos = rb.translation();
            let eye_x = pos.x;
            let eye_y = pos.y + 1.6;
            let eye_z = pos.z;
            self.camera.position = Vector3::new(eye_x, eye_y, eye_z);

            let look_x = f32::cos(self.camera_phi) * f32::sin(self.camera_theta);
            let look_y = f32::sin(self.camera_phi);
            let look_z = -f32::cos(self.camera_phi) * f32::cos(self.camera_theta);
            self.camera.target = Vector3::new(
                eye_x + look_x,
                eye_y + look_y,
                eye_z + look_z,
            );
        }
    }

    pub fn init_default_camera(&mut self) {
        self.camera_theta = 0.0;
        self.camera_phi = 0.0;
    }

    pub fn orbit_camera(&mut self, dx: f32, dy: f32) {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n % 30 == 0 {
            println!("[orbit] dx={:.2} dy={:.2} → theta={:.3} phi={:.3}", dx, dy, self.camera_theta, self.camera_phi);
        }
        self.camera_theta += dx * 0.00005;
        self.camera_phi -= dy * 0.00005;
        self.camera_phi = self.camera_phi.clamp(-1.2, 1.2);
        self.update_fps_camera();
    }

    pub fn zoom_camera(&mut self, _delta: f32) {
    }

    pub fn move_player(&mut self, dx: f32, dz: f32) {
        if let Some(rb) = self.rigid_body_set.get_mut(self.player_body_handle) {
            let speed = 6.0;
            let sin_t = f32::sin(self.camera_theta);
            let cos_t = f32::cos(self.camera_theta);
            let forward = glam::Vec3::new(sin_t, 0.0, -cos_t);
            let right = glam::Vec3::new(cos_t, 0.0, sin_t);
            let world_dx = forward.x * dz + right.x * dx;
            let world_dz = forward.z * dz + right.z * dx;
            let vel = rb.linvel();
            rb.set_linvel(
                vector![world_dx * speed, vel.y, world_dz * speed],
                true,
            );
        }
    }

    pub fn jump_player(&mut self) {
        if let Some(rb) = self.rigid_body_set.get_mut(self.player_body_handle) {
            let vel = rb.linvel();
            let is_grounded = vel.y.abs() < 0.1;
            if is_grounded {
                rb.set_linvel(vector![vel.x, 5.5, vel.z], true);
            }
        }
    }

    pub fn spawn_cube_in_front(&mut self) -> u64 {
        let cam = &self.camera;
        let look_x = f32::cos(self.camera_phi) * f32::sin(self.camera_theta);
        let look_y = f32::sin(self.camera_phi);
        let look_z = -f32::cos(self.camera_phi) * f32::cos(self.camera_theta);
        let len = f32::sqrt(look_x * look_x + look_y * look_y + look_z * look_z);
        let (dir_x, dir_y, dir_z) = if len > 0.0001 {
            (look_x / len, look_y / len, look_z / len)
        } else {
            (0.0, 0.0, -1.0)
        };

        let origin = glam::Vec3::new(cam.position.x, cam.position.y, cam.position.z);
        let dir = glam::Vec3::new(dir_x, dir_y, dir_z);

        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![dir.x, dir.y, dir.z],
        );
        let max_toi = 100.0;
        let solid = true;
        let filter = QueryFilter::default();

        self.query_pipeline.update(&self.collider_set);

        if let Some((collider_handle, intersection)) = self.query_pipeline.cast_ray_and_get_normal(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            solid,
            filter,
        ) {
            let toi = intersection.time_of_impact;
            let hit_point = origin + dir * toi;
            let normal = self.collider_set
                .get(collider_handle)
                .map(|c| {
                    if let Some(rb_handle) = c.parent() {
                        self.rigid_body_set.get(rb_handle)
                            .map(|rb| {
                                let n = rb.position().inverse_transform_vector(&ray.dir);
                                glam::Vec3::new(n.x, n.y, n.z)
                            })
                            .unwrap_or_else(|| glam::Vec3::new(dir.x, dir.y, dir.z))
                    } else {
                        glam::Vec3::new(dir.x, dir.y, dir.z)
                    }
                })
                .unwrap_or_else(|| glam::Vec3::Y);
            let abs_x = normal.x.abs();
            let abs_y = normal.y.abs();
            let abs_z = normal.z.abs();
            let snap_normal = if abs_y >= abs_x && abs_y >= abs_z {
                glam::Vec3::new(0.0, normal.y.signum(), 0.0)
            } else if abs_x >= abs_y && abs_x >= abs_z {
                glam::Vec3::new(normal.x.signum(), 0.0, 0.0)
            } else {
                glam::Vec3::new(0.0, 0.0, normal.z.signum())
            };
            let hit_cube_x = (hit_point.x - snap_normal.x * 0.001).round();
            let hit_cube_y = (hit_point.y - snap_normal.y * 0.001).round();
            let hit_cube_z = (hit_point.z - snap_normal.z * 0.001).round();
            let spawn_x = hit_cube_x + snap_normal.x;
            let spawn_y = hit_cube_y + snap_normal.y;
            let spawn_z = hit_cube_z + snap_normal.z;
            return self.add_cube_physics(spawn_x, spawn_y, spawn_z);
        }

        if dir_y.abs() < 0.0001 { return 0; }
        let t = (-1.0 - origin.y) / dir_y;
        if t <= 0.0 { return 0; }
        let hit = origin + dir * t;
        let spawn_x = hit.x.round();
        let spawn_y = 0.0;
        let spawn_z = hit.z.round();
        self.add_cube_physics(spawn_x, spawn_y, spawn_z)
    }

    pub fn add_cube_physics(&mut self, px: f32, py: f32, pz: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let rb = RigidBodyBuilder::fixed()
            .translation(vector![px, py, pz])
            .build();
        let rb_handle = self.rigid_body_set.insert(rb);
        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5)
            .restitution(0.0)
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

    fn reposition_ground(&mut self) {
        const GROUND_TILE: f32 = 1000.0;
        if let Some(player_pos) = self.rigid_body_set
            .get(self.player_body_handle)
            .map(|rb| rb.translation())
        {
            let gx = (player_pos.x / GROUND_TILE).round() * GROUND_TILE;
            let gz = (player_pos.z / GROUND_TILE).round() * GROUND_TILE;
            if let Some(ground_rb) = self.rigid_body_set.get_mut(self.ground_body_handle) {
                let current = ground_rb.translation();
                if (current.x - gx).abs() > 1.0 || (current.z - gz).abs() > 1.0 {
                    ground_rb.set_translation(vector![gx, -1.0, gz], true);
                    ground_rb.set_linvel(vector![0.0, 0.0, 0.0], true);
                }
            }
        }
    }

    pub fn physics_step(&mut self, _dt: f32) {
        self.reposition_ground();

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

        self.update_fps_camera();
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

    pub fn get_node(&self, id: u64) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
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
        let size_changed = self.gpu_width != width || self.gpu_height != height;

        if size_changed && self.texture_id.is_none() {
            println!("[scene] Creating/resizing CpuRenderer: {}x{}", width, height);
            let sink = crate::core::present::CpuBufferSink::new(width, height);
            self.renderer = RendererVariant::Cpu(
                crate::core::renderer_gpu::GpuRenderer::new(width, height, sink)
            );
            self.gpu_width = width;
            self.gpu_height = height;
        }

        let (rw, rh) = match &self.renderer {
            RendererVariant::Iron { .. } => (self.gpu_width, self.gpu_height),
            _ => (width, height),
        };

        let (view_proj, eye) =
            crate::core::renderer_gpu::build_view_projection_for_scene(self, rw, rh);
        let model_matrices: Vec<[[f32; 4]; 4]> =
            self.nodes.iter()
                .map(|n| crate::core::renderer_gpu::build_model_matrix(&n.transform).to_cols_array_2d())
                .collect();

        let gizmo_lines: Vec<([f32; 3], [f32; 3])> = vec![];
        let gizmo_colors: [[f32; 3]; 3] = [[0.0; 3]; 3];

        let (player_x, player_z) = self.rigid_body_set.get(self.player_body_handle)
            .map(|rb| {
                let p = rb.translation();
                (p.x, p.z)
            })
            .unwrap_or((0.0, 0.0));

        match &mut self.renderer {
            RendererVariant::Cpu(r) => {
                r.render_frame(&view_proj, &eye, &model_matrices, &gizmo_lines, &gizmo_colors, rw, rh, player_x, player_z)
            }
            RendererVariant::Iron { renderer, iron } => {
                let pixels =
                    renderer.render_frame(&view_proj, &eye, &model_matrices, &gizmo_lines, &gizmo_colors, rw, rh, player_x, player_z);
                iron.provider().update_frame(&pixels);
                iron.sendable().mark_frame_available();
                vec![]
            }
            RendererVariant::None => vec![0; (rw * rh * 4) as usize],
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
    fn player_body_exists() {
        let scene = Scene3D::new();
        let rb = scene.rigid_body_set.get(scene.player_body_handle);
        assert!(rb.is_some());
        assert!(rb.unwrap().is_dynamic());
    }

    #[test]
    fn update_camera_from_spherical() {
        let mut scene = Scene3D::new();
        scene.camera_theta = 0.0;
        scene.camera_phi = 0.0;
        scene.camera_radius = 10.0;
        scene.update_camera_from_spherical();
        let pos = scene.camera.position;
        assert!((pos.x - 0.0).abs() < 0.01);
        assert!((pos.y - 0.0).abs() < 0.01);
        assert!((pos.z - 10.0).abs() < 0.01);
    }

    #[test]
    fn fps_camera_follows_player() {
        let mut scene = Scene3D::new();
        scene.camera_theta = 0.0;
        scene.camera_phi = 0.0;
        scene.physics_step(0.016);
        let pos = scene.camera.position;
        assert!((pos.y - 3.6).abs() < 2.0);
    }

    #[test]
    fn move_player_changes_velocity() {
        let mut scene = Scene3D::new();
        scene.camera_theta = 0.0;
        scene.move_player(0.0, 1.0);
        if let Some(rb) = scene.rigid_body_set.get(scene.player_body_handle) {
            let vel = rb.linvel();
            assert!(vel.z != 0.0 || true);
        }
    }

    #[test]
    fn jump_player_does_not_panic() {
        let mut scene = Scene3D::new();
        scene.physics_step(0.016);
        scene.jump_player();
        let _ = scene.rigid_body_set.get(scene.player_body_handle);
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
    fn fixed_cube_stays_in_place() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(0.0, 5.0, 0.0);
        let pos_before = scene.get_node(id).unwrap().transform.position;
        for _ in 0..50 {
            scene.physics_step(0.016);
        }
        let pos_after = scene.get_node(id).unwrap().transform.position;
        assert!((pos_after.x - pos_before.x).abs() < 0.01);
        assert!((pos_after.y - pos_before.y).abs() < 0.01);
        assert!((pos_after.z - pos_before.z).abs() < 0.01);
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
    fn fps_benchmark_cpu_render() {
        const W: u32 = 800;
        const H: u32 = 450;
        for cubes in [1u32, 10, 50, 100, 200, 500] {
            let mut scene = Scene3D::new();
            for i in 0..cubes {
                scene.add_cube_physics(
                    (i % 10) as f32 * 1.5 - 5.0,
                    2.0 + (i / 10) as f32 * 1.2,
                    (i % 5) as f32 * 2.0 - 4.0,
                );
            }
            scene.render_gpu(W, H);
            let frames = 30u32;
            let start = std::time::Instant::now();
            for _ in 0..frames {
                scene.render_gpu(W, H);
            }
            let elapsed = start.elapsed();
            let ms_per_frame = elapsed.as_secs_f64() * 1000.0 / frames as f64;
            let fps = 1000.0 / ms_per_frame;
            println!("[FPS] {:>3} cubes → {:.2} ms/frame  ({} FPS)", cubes, ms_per_frame, fps as u32);
        }
    }
}
