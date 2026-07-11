use crate::core::math::Vector3;
use crate::core::scene::{Camera, Node, Scene3D};

pub fn render_scene(scene: &Scene3D, width: u32, height: u32) -> Vec<u8> {
    let size = (width as usize) * (height as usize) * 4;
    let mut buffer = vec![0u8; size];
    clear_buffer(&mut buffer, width, height, 30, 30, 40);

    for node in &scene.nodes {
        if node.mesh_id == Some(100) {
            draw_wireframe_cube(&mut buffer, width, height, node, &scene.camera);
        }
    }

    buffer
}

fn clear_buffer(buffer: &mut [u8], width: u32, height: u32, r: u8, g: u8, b: u8) {
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) as usize) * 4;
            buffer[idx]     = r;
            buffer[idx + 1] = g;
            buffer[idx + 2] = b;
            buffer[idx + 3] = 255;
        }
    }
}

#[allow(dead_code)]
fn set_pixel(buffer: &mut [u8], width: u32, x: i32, y: i32, r: u8, g: u8, b: u8) {
    if x < 0 || y < 0 || x >= width as i32 || y >= width as i32 {
        return;
    }
    if x >= width as i32 || y >= width as i32 {
        return;
    }
    let idx = ((y as u32 * width + x as u32) as usize) * 4;
    buffer[idx]     = r;
    buffer[idx + 1] = g;
    buffer[idx + 2] = b;
    buffer[idx + 3] = 255;
}

#[allow(dead_code)]
fn set_pixel_u32(buffer: &mut [u8], width: u32, x: u32, y: u32, r: u8, g: u8, b: u8) {
    if x >= width || y >= width {
        return;
    }
    let idx = ((y * width + x) as usize) * 4;
    buffer[idx]     = r;
    buffer[idx + 1] = g;
    buffer[idx + 2] = b;
    buffer[idx + 3] = 255;
}

fn draw_line(buffer: &mut [u8], width: u32, height: u32, x0: i32, y0: i32, x1: i32, y1: i32, r: u8, g: u8, b: u8) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0;
    let mut cy = y0;
    let max_w = width as i32;
    let max_h = height as i32;

    loop {
        if cx >= 0 && cx < max_w && cy >= 0 && cy < max_h {
            let idx = ((cy as u32 * width + cx as u32) as usize) * 4;
            buffer[idx]     = r;
            buffer[idx + 1] = g;
            buffer[idx + 2] = b;
            buffer[idx + 3] = 255;
        }
        if cx == x1 && cy == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; cx += sx; }
        if e2 <= dx { err += dx; cy += sy; }
    }
}

fn project_point(point: &Vector3, camera: &Camera, screen_width: u32, screen_height: u32) -> Option<(i32, i32)> {
    let forward = (camera.target - camera.position).normalize_or_zero();
    let world_up = Vector3::UP;
    let right = forward.dot(&world_up);
    let right_vec;
    let up_vec;
    if right.abs() > 0.999 {
        right_vec = Vector3::new(1.0, 0.0, 0.0);
        up_vec = Vector3::new(0.0, 0.0, -1.0);
    } else {
        right_vec = {
            let cr = Vector3::new(
                forward.y * world_up.z - forward.z * world_up.y,
                forward.z * world_up.x - forward.x * world_up.z,
                forward.x * world_up.y - forward.y * world_up.x,
            );
            cr.normalize_or_zero()
        };
        up_vec = {
            let cr = Vector3::new(
                right_vec.y * forward.z - right_vec.z * forward.y,
                right_vec.z * forward.x - right_vec.x * forward.z,
                right_vec.x * forward.y - right_vec.y * forward.x,
            );
            cr.normalize_or_zero()
        };
    }

    let rel = *point - camera.position;
    let cam_z = rel.dot(&forward);
    if cam_z <= 0.001 {
        return None;
    }

    let cam_x = rel.dot(&right_vec);
    let cam_y = rel.dot(&up_vec);

    let aspect = screen_width as f32 / screen_height as f32;
    let f = 1.0 / (camera.fov / 2.0).tan();
    let sx = screen_width as f32 / 2.0;
    let sy = screen_height as f32 / 2.0;

    let ndc_x = (cam_x / cam_z) * f;
    let ndc_y = -(cam_y / cam_z) * f * aspect;

    let px = (ndc_x * sx + sx) as i32;
    let py = (ndc_y * sy + sy) as i32;

    Some((px, py))
}

const CUBE_VERTICES: [(f32, f32, f32); 8] = [
    (-0.5, -0.5, -0.5),
    ( 0.5, -0.5, -0.5),
    ( 0.5,  0.5, -0.5),
    (-0.5,  0.5, -0.5),
    (-0.5, -0.5,  0.5),
    ( 0.5, -0.5,  0.5),
    ( 0.5,  0.5,  0.5),
    (-0.5,  0.5,  0.5),
];

const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0),
    (4, 5), (5, 6), (6, 7), (7, 4),
    (0, 4), (1, 5), (2, 6), (3, 7),
];

fn draw_wireframe_cube(buffer: &mut [u8], width: u32, height: u32, node: &Node, camera: &Camera) {
    let t = &node.transform;
    let mut projected: [Option<(i32, i32)>; 8] = [None; 8];

    for (i, &(vx, vy, vz)) in CUBE_VERTICES.iter().enumerate() {
        let local = Vector3::new(vx * t.scale.x, vy * t.scale.y, vz * t.scale.z);
        let ry = crate::core::math::Matrix4::from_rotation_y(t.rotation.y);
        let rx = crate::core::math::Matrix4::from_rotation_y(t.rotation.x);
        let rotated = ry.multiply(&rx).transform_vector3(&local);
        let world = rotated + t.position;
        projected[i] = project_point(&world, camera, width, height);
    }

    let edge_color = (0, 255, 64);
    for &(a, b) in &CUBE_EDGES {
        if let (Some((ax, ay)), Some((bx, by))) = (projected[a], projected[b]) {
            draw_line(buffer, width, height, ax, ay, bx, by, edge_color.0, edge_color.1, edge_color.2);
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scene::Scene3D;

    #[test]
    fn clear_buffer_fills_all_pixels() {
        let w = 10u32;
        let h = 8u32;
        let size = (w as usize) * (h as usize) * 4;
        let mut buf = vec![0u8; size];
        clear_buffer(&mut buf, w, h, 50, 100, 150);
        for y in 0..h {
            for x in 0..w {
                let idx = ((y * w + x) as usize) * 4;
                assert_eq!(buf[idx], 50, "R mismatch at ({},{})", x, y);
                assert_eq!(buf[idx + 1], 100);
                assert_eq!(buf[idx + 2], 150);
                assert_eq!(buf[idx + 3], 255);
            }
        }
    }

    #[test]
    fn render_scene_returns_correctly_sized_buffer() {
        let mut scene = Scene3D::new();
        scene.add_test_cube();
        let buf = render_scene(&scene, 64, 48);
        assert_eq!(buf.len(), 64 * 48 * 4);
    }

    #[test]
    fn project_point_behind_camera_returns_none() {
        let camera = Camera::default();
        let point = Vector3::new(0.0, 2.0, 10.0); // behind camera (cam looks toward 0,0,0)
        let result = project_point(&point, &camera, 64, 48);
        assert!(result.is_none());
    }

    #[test]
    fn project_point_in_front_returns_some() {
        let camera = Camera::default();
        let point = Vector3::new(0.0, 0.0, 0.0); // where camera looks
        let result = project_point(&point, &camera, 64, 48);
        assert!(result.is_some());
    }

    #[test]
    fn draw_line_sets_pixels() {
        let w = 10u32;
        let h = 10u32;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        draw_line(&mut buf, w, h, 0, 0, 9, 0, 255, 255, 255);
        for x in 0..10 {
            let idx = (x * 4) as usize;
            assert_eq!(buf[idx], 255);
            assert_eq!(buf[idx + 1], 255);
            assert_eq!(buf[idx + 2], 255);
        }
    }

    #[test]
    fn render_scene_with_camera_moved() {
        let mut scene = Scene3D::new();
        scene.add_test_cube();
        scene.camera.position = Vector3::new(0.0, 0.0, 3.0);
        let buf = render_scene(&scene, 100, 100);
        assert_eq!(buf.len(), 100 * 100 * 4);
    }
}
