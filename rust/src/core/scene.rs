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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoAxis { X, Y, Z }

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

    // camera spherical state
    pub camera_theta:  f32,
    pub camera_phi:    f32,
    pub camera_radius: f32,

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

    // selection & gizmo
    pub selected_node_id: Option<u64>,
    pub active_gizmo_axis: Option<GizmoAxis>,
    pub dragged_body:      Option<(RigidBodyHandle, f32)>,
    drag_gizmo_start:      f32,
    drag_node_id:          Option<u64>,
}

impl Scene3D {
    pub fn new() -> Self {
        let gravity = Vector3::new(0.0, -9.81, 0.0);

        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

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
            camera_theta:  0.45,
            camera_phi:    0.35,
            camera_radius: 7.0,
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
            selected_node_id:       None,
            active_gizmo_axis:      None,
            dragged_body:           None,
            drag_gizmo_start:       0.0,
            drag_node_id:           None,
        }
    }

    pub fn update_camera_from_spherical(&mut self) {
        let x = self.camera_radius * f32::cos(self.camera_phi) * f32::sin(self.camera_theta);
        let y = self.camera_radius * f32::sin(self.camera_phi);
        let z = self.camera_radius * f32::cos(self.camera_phi) * f32::cos(self.camera_theta);
        self.camera.position = Vector3::new(x, y, z);
        self.camera.target = Vector3::ZERO;
    }

    pub fn init_default_camera(&mut self) {
        self.camera_theta = 0.45;
        self.camera_phi = 0.35;
        self.camera_radius = 7.0;
        self.update_camera_from_spherical();
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

    fn point_line_distance(point: glam::Vec3, line_a: glam::Vec3, line_b: glam::Vec3) -> f32 {
        let ab = line_b - line_a;
        let ap = point - line_a;
        let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
        let closest = line_a + ab * t;
        (point - closest).length()
    }

    pub fn selected_node_position(&self) -> Option<glam::Vec3> {
        let node_id = self.selected_node_id?;
        let node = self.get_node(node_id)?;
        Some(glam::Vec3::new(
            node.transform.position.x,
            node.transform.position.y,
            node.transform.position.z,
        ))
    }

    pub fn handle_pointer_down(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> bool {
        let (origin, dir) = self.build_ray(screen_x, screen_y, screen_width, screen_height);

        // 1. If selected node exists, check gizmo arrow clicks
        if let Some(pos) = self.selected_node_position() {
            let gizmo_length = 1.5;
            let x_end = pos + glam::Vec3::X * gizmo_length;
            let y_end = pos + glam::Vec3::Y * gizmo_length;
            let z_end = pos + glam::Vec3::Z * gizmo_length;

            // Find closest point on each axis line to the ray
            let threshold: f32 = 0.25;
            // Actually use ray-line distance by checking closest approach
            let ray_dir = dir;
            let ray_origin = origin;

            // Ray-line distance: project ray onto line plane
            let axis_dist = |axis_end: glam::Vec3| -> f32 {
                let line_dir = (axis_end - pos).normalize();
                let w0 = ray_origin - pos;
                let a = ray_dir.dot(ray_dir);
                let b = ray_dir.dot(line_dir);
                let c = line_dir.dot(line_dir);
                let d = ray_dir.dot(w0);
                let e = line_dir.dot(w0);
                let denom = a * c - b * b;
                if denom.abs() < 0.0001 { return f32::MAX; }
                let sc = (b * e - c * d) / denom;
                let tc = (a * e - b * d) / denom;
                let tc = tc.clamp(0.0, gizmo_length);
                let closest_line = pos + line_dir * tc;
                let closest_ray = ray_origin + ray_dir * sc;
                (closest_line - closest_ray).length()
            };

            if axis_dist(x_end) < threshold {
                self.active_gizmo_axis = Some(GizmoAxis::X);
                if let Some(node_id) = self.selected_node_id {
                    if let Some(node) = self.get_node(node_id) {
                        self.drag_gizmo_start = node.transform.position.x;
                    }
                }
                return true;
            }
            if axis_dist(y_end) < threshold {
                self.active_gizmo_axis = Some(GizmoAxis::Y);
                if let Some(node_id) = self.selected_node_id {
                    if let Some(node) = self.get_node(node_id) {
                        self.drag_gizmo_start = node.transform.position.y;
                    }
                }
                return true;
            }
            if axis_dist(z_end) < threshold {
                self.active_gizmo_axis = Some(GizmoAxis::Z);
                if let Some(node_id) = self.selected_node_id {
                    if let Some(node) = self.get_node(node_id) {
                        self.drag_gizmo_start = node.transform.position.z;
                    }
                }
                return true;
            }
        }

        // 2. Check body ray hit
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
                let is_dynamic = self.rigid_body_set.get(rb_handle).map_or(false, |rb| rb.is_dynamic());
                if !is_dynamic {
                    // ground — ignore
                    self.selected_node_id = None;
                    self.active_gizmo_axis = None;
                    return false;
                }

                // dynamic cube — select and start drag
                self.selected_node_id = self.nodes.iter()
                    .find(|n| n.rb_handle == Some(rb_handle))
                    .map(|n| n.id);

                let current_y = self.rigid_body_set.get(rb_handle).map_or(0.0, |rb| rb.translation().y);
                self.dragged_body = Some((rb_handle, current_y));
                self.drag_node_id = self.nodes.iter()
                    .find(|n| n.rb_handle == Some(rb_handle))
                    .map(|n| n.id);
                return true;
            }
        }

        // hit nothing — deselect
        self.selected_node_id = None;
        self.active_gizmo_axis = None;
        false
    }

    pub fn handle_pointer_move(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        if let Some(axis) = self.active_gizmo_axis {
            let (origin, dir) = self.build_ray(screen_x, screen_y, screen_width, screen_height);

            let node_id = match self.drag_node_id {
                Some(id) => id,
                None => return,
            };

            let pos = self.get_node(node_id).map(|n| n.transform.position).unwrap_or(Vector3::ZERO);
            let pos = glam::Vec3::new(pos.x, pos.y, pos.z);

            // Build plane perpendicular to axis, pass through node position
            let (plane_normal, plane_point) = match axis {
                GizmoAxis::X => (glam::Vec3::Y, pos),
                GizmoAxis::Y => (glam::Vec3::X, pos),
                GizmoAxis::Z => (glam::Vec3::Y, pos),
            };

            let denom = plane_normal.dot(dir);
            if denom.abs() < 0.0001 { return; }
            let t = (plane_point - origin).dot(plane_normal) / denom;
            let intersection = origin + dir * t;

            let handle = match self.drag_node_id.and_then(|id| self.get_node(id)).and_then(|n| n.rb_handle) {
                Some(h) => h,
                None => return,
            };

            let axis_val = match axis {
                GizmoAxis::X => intersection.x,
                GizmoAxis::Y => intersection.y,
                GizmoAxis::Z => intersection.z,
            };

            if let Some(rb) = self.rigid_body_set.get_mut(handle) {
                let mut trans = *rb.translation();
                match axis {
                    GizmoAxis::X => trans.x = axis_val.clamp(-6.0, 6.0),
                    GizmoAxis::Y => trans.y = axis_val,
                    GizmoAxis::Z => trans.z = axis_val.clamp(-6.0, 6.0),
                }
                rb.set_linvel(vector![0.0, 0.0, 0.0], true);
                rb.set_angvel(vector![0.0, 0.0, 0.0], true);
                rb.set_translation(trans, true);
            }
            return;
        }

        // plain drag (no gizmo axis active)
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
        self.active_gizmo_axis = None;
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

        let gizmo_lines: Vec<([f32; 3], [f32; 3])> = self
            .selected_node_position()
            .map(|p| {
                let l = 1.5;
                vec![
                    ([p.x, p.y, p.z], [p.x + l, p.y, p.z]),
                    ([p.x, p.y, p.z], [p.x, p.y + l, p.z]),
                    ([p.x, p.y, p.z], [p.x, p.y, p.z + l]),
                ]
            })
            .unwrap_or_default();

        // gizmo colors: X=red-ish, Y=green, Z=blue
        let gizmo_colors: [[f32; 3]; 3] = [
            [1.0, 0.2, 0.2],
            [0.2, 1.0, 0.2],
            [0.2, 0.3, 1.0],
        ];

        match &mut self.renderer {
            RendererVariant::Cpu(r) => {
                r.render_frame(&view_proj, &eye, &node_transforms, &gizmo_lines, &gizmo_colors, width, height)
            }
            RendererVariant::Iron { renderer, iron } => {
                let pixels =
                    renderer.render_frame(&view_proj, &eye, &node_transforms, &gizmo_lines, &gizmo_colors, width, height);
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
        assert!(y_after < y_before);
    }

    #[test]
    fn physics_step_cube_doesnt_fall_through_ground() {
        let mut scene = Scene3D::new();
        let id = scene.add_cube_physics(0.0, 5.0, 0.0);
        for _ in 0..300 {
            scene.physics_step(0.016);
        }
        let y = scene.get_node(id).unwrap().transform.position.y;
        assert!(y >= -0.6);
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
}
