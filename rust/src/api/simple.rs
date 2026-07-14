use crate::core::scene::Scene3D;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn create_scene() -> Scene3D {
    Scene3D::new()
}

pub fn add_cube(
    scene: &mut Scene3D,
    x: f32, y: f32, z: f32,
    r: f32, g: f32, b: f32,
) -> u64 {
    scene.add_cube_physics(x, y, z, [r, g, b])
}

#[flutter_rust_bridge::frb(sync)]
pub fn physics_step(scene: &mut Scene3D, dt: f32) {
    scene.physics_step(dt);
}

#[flutter_rust_bridge::frb(sync)]
pub fn init_default_camera(scene: &mut Scene3D) {
    scene.init_default_camera();
}

#[flutter_rust_bridge::frb(sync)]
pub fn orbit_camera(scene: &mut Scene3D, dx: f32, dy: f32) {
    scene.orbit_camera(dx, dy);
}

#[flutter_rust_bridge::frb(sync)]
pub fn zoom_camera(scene: &mut Scene3D, delta: f32) {
    scene.zoom_camera(delta);
}

#[flutter_rust_bridge::frb(sync)]
pub fn move_player(scene: &mut Scene3D, dx: f32, dz: f32) {
    scene.move_player(dx, dz);
}

#[flutter_rust_bridge::frb(sync)]
pub fn jump_player(scene: &mut Scene3D) {
    scene.jump_player();
}

#[flutter_rust_bridge::frb(sync)]
pub fn spawn_cube_in_front(scene: &mut Scene3D, r: f32, g: f32, b: f32) -> u64 {
    scene.spawn_cube_in_front(r, g, b)
}

#[flutter_rust_bridge::frb(sync)]
pub fn destroy_looked_block(scene: &mut Scene3D) -> bool {
    scene.destroy_looked_block()
}

#[flutter_rust_bridge::frb(sync)]
pub fn spawn_fluid_at(scene: &mut Scene3D, x: f32, y: f32, z: f32) {
    scene.spawn_fluid_at(x, y, z);
}

pub fn render_scene(scene: &mut Scene3D, width: u32, height: u32) -> Vec<u8> {
    scene.render_gpu(width, height)
}

#[flutter_rust_bridge::frb(sync)]
pub fn update_node_transform(scene: &mut Scene3D, node_id: u64, px: f32, py: f32, pz: f32, rx: f32, ry: f32, rz: f32, sx: f32, sy: f32, sz: f32) {
    scene.update_node_transform(node_id, px, py, pz, rx, ry, rz, sx, sy, sz);
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

pub fn set_camera_fov(scene: &mut Scene3D, fov: f32) {
    scene.camera.fov = fov;
}

pub fn init_native_texture(scene: &mut Scene3D, engine_handle: i64, width: u32, height: u32) -> i64 {
    scene.init_native_texture(engine_handle, width, height)
}

pub fn render_native_frame(scene: &mut Scene3D, width: u32, height: u32) {
    scene.render_gpu(width, height);
}
