// File: crates/storm-math/src/lib.rs
// Math utilities and types for StormCore
// Provides vector math, transforms, and spatial calculations

use serde::{Deserialize, Serialize};

pub mod vector;
pub mod quaternion;
pub mod transform;
pub mod spatial;

pub use vector::*;
pub use quaternion::*;
pub use transform::*;
pub use spatial::*;

// Re-export glam types with different names to avoid conflicts
pub use glam::{Vec2 as GlamVec2, Vec3 as GlamVec3, Vec4 as GlamVec4, Mat3, Mat4, Quat as GlamQuat};

/// Custom 3D Vector with serde support
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const X: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            self
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Convert to glam Vec3 for interoperability
    pub fn to_glam(self) -> GlamVec3 {
        GlamVec3::new(self.x, self.y, self.z)
    }

    /// Create from glam Vec3
    pub fn from_glam(vec: GlamVec3) -> Self {
        Self { x: vec.x, y: vec.y, z: vec.z }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(vec: Vec3) -> Self {
        [vec.x, vec.y, vec.z]
    }
}

/// Custom Quaternion for rotations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();

        Self {
            x: axis.x * sin_half,
            y: axis.y * sin_half,
            z: axis.z * sin_half,
            w: cos_half,
        }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            self
        }
    }

    pub fn slerp(self, other: Self, t: f32) -> Self {
        // Spherical linear interpolation
        let dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;

        if dot.abs() > 0.9995 {
            // Linear interpolation for very close quaternions
            let result = Self {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            };
            return result.normalize();
        }

        let theta_0 = dot.abs().acos();
        let sin_theta_0 = theta_0.sin();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let s0 = cos_theta - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        Self {
            x: s0 * self.x + s1 * other.x,
            y: s0 * self.y + s1 * other.y,
            z: s0 * self.z + s1 * other.z,
            w: s0 * self.w + s1 * other.w,
        }
    }

    /// Convert to glam Quat for interoperability
    pub fn to_glam(self) -> GlamQuat {
        GlamQuat::from_xyzw(self.x, self.y, self.z, self.w)
    }

    /// Create from glam Quat
    pub fn from_glam(quat: GlamQuat) -> Self {
        Self { x: quat.x, y: quat.y, z: quat.z, w: quat.w }
    }
}

impl From<[f32; 4]> for Quat {
    fn from(arr: [f32; 4]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
            w: arr[3],
        }
    }
}

impl From<Quat> for [f32; 4] {
    fn from(quat: Quat) -> Self {
        [quat.x, quat.y, quat.z, quat.w]
    }
}

/// 3D Transform combining position, rotation, and scale
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { position, rotation, scale }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    pub fn transform_point(self, point: Vec3) -> Vec3 {
        // Apply scale, rotation, then translation
        let scaled = Vec3::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
            point.z * self.scale.z,
        );

        // Apply rotation (simplified - would need proper quaternion rotation)
        let rotated = scaled; // Placeholder

        // Apply translation
        rotated + self.position
    }

    pub fn inverse(self) -> Self {
        // Simplified inverse transform
        Self {
            position: Vec3::new(-self.position.x, -self.position.y, -self.position.z),
            rotation: Quat::new(-self.rotation.x, -self.rotation.y, -self.rotation.z, self.rotation.w),
            scale: Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let sum = a + b;
        assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));

        let diff = b - a;
        assert_eq!(diff, Vec3::new(3.0, 3.0, 3.0));

        let scaled = a * 2.0;
        assert_eq!(scaled, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_vec3_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);

        let normalized = v.normalize();
        assert!((normalized.length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_identity() {
        let q = Quat::IDENTITY;
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 0.0);
        assert_eq!(q.y, 0.0);
        assert_eq!(q.z, 0.0);
    }

    #[test]
    fn test_transform_default() {
        let t = Transform::default();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }
}