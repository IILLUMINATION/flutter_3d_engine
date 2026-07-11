use crate::core::scene::Scene3D;
use crate::core::math::Vector3;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn create_scene() -> Scene3D {
    let mut scene = Scene3D::new();
    scene.add_test_cube();
    scene
}

pub fn update_scene(scene: &mut Scene3D, dt: f32) {
    scene.update(dt);
}

pub fn get_camera_position(scene: &Scene3D) -> (f32, f32, f32) {
    let cam = &scene.camera;
    (cam.position.x, cam.position.y, cam.position.z)
}

pub fn get_camera_target(scene: &Scene3D) -> (f32, f32, f32) {
    let cam = &scene.camera;
    (cam.target.x, cam.target.y, cam.target.z)
}

pub fn get_camera_fov(scene: &Scene3D) -> f32 {
    scene.camera.fov
}

pub fn add_cube(scene: &mut Scene3D) -> u64 {
    scene.add_test_cube()
}

pub fn add_node(
    scene: &mut Scene3D,
    px: f32, py: f32, pz: f32,
    rx: f32, ry: f32, rz: f32,
    sx: f32, sy: f32, sz: f32,
    mesh_id: Option<u64>,
) -> u64 {
    let transform = crate::core::math::Transform {
        position: Vector3::new(px, py, pz),
        rotation: Vector3::new(rx, ry, rz),
        scale:    Vector3::new(sx, sy, sz),
    };
    scene.add_node(transform, mesh_id)
}

pub fn get_node_position(scene: &Scene3D, node_id: u64) -> Option<(f32, f32, f32)> {
    scene.get_node(node_id).map(|n| {
        (n.transform.position.x, n.transform.position.y, n.transform.position.z)
    })
}

pub fn get_node_rotation(scene: &Scene3D, node_id: u64) -> Option<(f32, f32, f32)> {
    scene.get_node(node_id).map(|n| {
        (n.transform.rotation.x, n.transform.rotation.y, n.transform.rotation.z)
    })
}

pub fn get_node_scale(scene: &Scene3D, node_id: u64) -> Option<(f32, f32, f32)> {
    scene.get_node(node_id).map(|n| {
        (n.transform.scale.x, n.transform.scale.y, n.transform.scale.z)
    })
}

pub fn get_elapsed_time(scene: &Scene3D) -> f32 {
    scene.elapsed
}

pub fn get_node_count(scene: &Scene3D) -> u64 {
    scene.nodes.len() as u64
}

pub fn set_camera_position(scene: &mut Scene3D, x: f32, y: f32, z: f32) {
    scene.camera.position = Vector3::new(x, y, z);
}

pub fn set_camera_target(scene: &mut Scene3D, x: f32, y: f32, z: f32) {
    scene.camera.target = Vector3::new(x, y, z);
}

pub fn set_camera_fov(scene: &mut Scene3D, fov: f32) {
    scene.camera.fov = fov;
}

pub fn render_scene(scene: &mut Scene3D, width: u32, height: u32) -> Vec<u8> {
    scene.render_gpu(width, height)
}

pub fn init_native_texture(scene: &mut Scene3D, engine_handle: i64, width: u32, height: u32) -> i64 {
    scene.init_native_texture(engine_handle, width, height)
}

pub fn render_native_frame(scene: &mut Scene3D, width: u32, height: u32) {
    scene.render_gpu(width, height);
}
