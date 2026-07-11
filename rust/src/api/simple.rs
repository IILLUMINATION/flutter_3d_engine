use crate::core::scene::Scene3D;
use crate::core::math::Vector3;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn create_scene() -> Scene3D {
    Scene3D::new()
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

pub fn add_cube(
    scene: &mut Scene3D,
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
) -> u64 {
    let _color = r * r + g * g + b * b;
    scene.add_cube_physics(x, y, z)
}

#[flutter_rust_bridge::frb(sync)]
pub fn update_node_transform(
    scene: &mut Scene3D,
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
    scene.update_node_transform(node_id, px, py, pz, rx, ry, rz, sx, sy, sz);
}

#[flutter_rust_bridge::frb(sync)]
pub fn update_camera(
    scene: &mut Scene3D,
    px: f32,
    py: f32,
    pz: f32,
    tx: f32,
    ty: f32,
    tz: f32,
) {
    scene.update_camera(px, py, pz, tx, ty, tz);
}

#[flutter_rust_bridge::frb(sync)]
pub fn physics_step(scene: &mut Scene3D, dt: f32) {
    scene.physics_step(dt);
}

#[flutter_rust_bridge::frb(sync)]
pub fn handle_pointer_down(
    scene: &mut Scene3D,
    screen_x: f32,
    screen_y: f32,
    screen_width: f32,
    screen_height: f32,
) -> bool {
    scene.handle_pointer_down(screen_x, screen_y, screen_width, screen_height)
}

#[flutter_rust_bridge::frb(sync)]
pub fn handle_pointer_move(
    scene: &mut Scene3D,
    screen_x: f32,
    screen_y: f32,
    screen_width: f32,
    screen_height: f32,
) {
    scene.handle_pointer_move(screen_x, screen_y, screen_width, screen_height);
}

#[flutter_rust_bridge::frb(sync)]
pub fn handle_pointer_up(scene: &mut Scene3D) {
    scene.handle_pointer_up();
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
