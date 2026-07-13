use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self  = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self   = Self { x: 0.0, y: 1.0, z: 0.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, rhs: &Vector3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize_or_zero(&self) -> Vector3 {
        let len = self.length();
        if len > f32::EPSILON {
            *self / len
        } else {
            Vector3::ZERO
        }
    }
}

impl Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self { x: self.x / scalar, y: self.y / scalar, z: self.z / scalar }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    pub data: [[f32; 4]; 4],
}

impl Matrix4 {
    pub const IDENTITY: Self = Self {
        data: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn from_translation(t: &Vector3) -> Self {
        let mut m = Self::IDENTITY;
        m.data[0][3] = t.x;
        m.data[1][3] = t.y;
        m.data[2][3] = t.z;
        m
    }

    pub fn from_scale(s: &Vector3) -> Self {
        let mut m = Self::IDENTITY;
        m.data[0][0] = s.x;
        m.data[1][1] = s.y;
        m.data[2][2] = s.z;
        m
    }

    pub fn from_rotation_x(angle_rad: f32) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let mut m = Self::IDENTITY;
        m.data[1][1] = c; m.data[1][2] = -s;
        m.data[2][1] = s; m.data[2][2] = c;
        m
    }

    pub fn from_rotation_y(angle_rad: f32) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let mut m = Self::IDENTITY;
        m.data[0][0] = c;
        m.data[0][2] = s;
        m.data[2][0] = -s;
        m.data[2][2] = c;
        m
    }

    pub fn from_rotation_z(angle_rad: f32) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let mut m = Self::IDENTITY;
        m.data[0][0] = c; m.data[0][1] = -s;
        m.data[1][0] = s; m.data[1][1] = c;
        m
    }

    pub fn multiply(&self, rhs: &Matrix4) -> Matrix4 {
        let mut out = Matrix4 { data: [[0.0; 4]; 4] };
        for row in 0..4 {
            for col in 0..4 {
                out.data[row][col] = self.data[row][0] * rhs.data[0][col]
                                   + self.data[row][1] * rhs.data[1][col]
                                   + self.data[row][2] * rhs.data[2][col]
                                   + self.data[row][3] * rhs.data[3][col];
            }
        }
        out
    }

    pub fn transform_vector3(&self, v: &Vector3) -> Vector3 {
        let x = self.data[0][0] * v.x + self.data[0][1] * v.y + self.data[0][2] * v.z + self.data[0][3];
        let y = self.data[1][0] * v.x + self.data[1][1] * v.y + self.data[1][2] * v.z + self.data[1][3];
        let z = self.data[2][0] * v.x + self.data[2][1] * v.y + self.data[2][2] * v.z + self.data[2][3];
        Vector3::new(x, y, z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        position: Vector3::ZERO,
        rotation: Vector3::ZERO,
        scale:    Vector3::ONE,
    };

    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self { position: Vector3::new(x, y, z), ..Self::IDENTITY }
    }

    pub fn apply_to_point(&self, point: &Vector3) -> Vector3 {
        let scaled = Vector3::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
            point.z * self.scale.z,
        );

        let rx = Matrix4::from_rotation_x(self.rotation.x);
        let ry = Matrix4::from_rotation_y(self.rotation.y);
        let rz = Matrix4::from_rotation_z(self.rotation.z);
        let rotated = rz.multiply(&ry).multiply(&rx).transform_vector3(&scaled);

        rotated + self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector3_add() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vector3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn vector3_sub() {
        let a = Vector3::new(5.0, 9.0, 1.0);
        let b = Vector3::new(2.0, 3.0, 0.0);
        assert_eq!(a - b, Vector3::new(3.0, 6.0, 1.0));
    }

    #[test]
    fn vector3_mul_scalar() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v * 2.0, Vector3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn vector3_div_scalar() {
        let v = Vector3::new(10.0, 20.0, 30.0);
        assert_eq!(v / 2.0, Vector3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn vector3_dot() {
        let a = Vector3::new(1.0, 0.0, 0.0);
        let b = Vector3::new(0.0, 1.0, 0.0);
        assert_eq!(a.dot(&b), 0.0);
        assert_eq!(a.dot(&a), 1.0);
    }

    #[test]
    fn vector3_length() {
        assert_eq!(Vector3::new(3.0, 4.0, 0.0).length(), 5.0);
    }

    #[test]
    fn vector3_normalize() {
        let v = Vector3::new(0.0, 5.0, 0.0);
        assert_eq!(v.normalize_or_zero(), Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn zero_vector_normalize_stays_zero() {
        let v = Vector3::ZERO;
        assert_eq!(v.normalize_or_zero(), Vector3::ZERO);
    }

    #[test]
    fn transform_identity_no_change() {
        let t = Transform::IDENTITY;
        let p = Vector3::new(10.0, 20.0, 30.0);
        assert_eq!(t.apply_to_point(&p), p);
    }

    #[test]
    fn transform_translation() {
        let t = Transform::from_position(5.0, 0.0, -3.0);
        let p = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(t.apply_to_point(&p), Vector3::new(6.0, 2.0, 0.0));
    }

    #[test]
    fn transform_scale() {
        let t = Transform { scale: Vector3::new(2.0, 3.0, 4.0), ..Transform::IDENTITY };
        let p = Vector3::new(1.0, 1.0, 1.0);
        assert_eq!(t.apply_to_point(&p), Vector3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn transform_rotation_y_90_deg() {
        let t = Transform {
            rotation: Vector3::new(0.0, std::f32::consts::FRAC_PI_2, 0.0),
            ..Transform::IDENTITY
        };
        let p = Vector3::new(1.0, 0.0, 0.0);
        let result = t.apply_to_point(&p);
        let epsilon = 1e-5;
        assert!((result.x - 0.0).abs() < epsilon, "x should be ~0, got {}", result.x);
        assert!((result.y - 0.0).abs() < epsilon, "y should be ~0");
        assert!((result.z + 1.0).abs() < epsilon, "z should be ~-1, got {}", result.z);
    }

    #[test]
    fn matrix4_identity_multiply() {
        let a = Matrix4::IDENTITY;
        let b = Matrix4::IDENTITY;
        let c = a.multiply(&b);
        for row in 0..4 {
            for col in 0..4 {
                assert_eq!(c.data[row][col], Matrix4::IDENTITY.data[row][col]);
            }
        }
    }

    #[test]
    fn matrix4_translation() {
        let m = Matrix4::from_translation(&Vector3::new(1.0, 2.0, 3.0));
        let v = Vector3::new(0.0, 0.0, 0.0);
        assert_eq!(m.transform_vector3(&v), Vector3::new(1.0, 2.0, 3.0));
    }
}
