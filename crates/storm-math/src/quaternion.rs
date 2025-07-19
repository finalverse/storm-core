// File: crates/storm-math/src/quaternion.rs
// Quaternion utilities

use crate::{Quat, Vec3};

impl Quat {
    /// Create quaternion from euler angles
    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        let (sx, cx) = (x * 0.5).sin_cos();
        let (sy, cy) = (y * 0.5).sin_cos();
        let (sz, cz) = (z * 0.5).sin_cos();

        Self {
            x: sx * cy * cz - cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
            w: cx * cy * cz + sx * sy * sz,
        }
    }

    /// Convert to euler angles
    pub fn to_euler(self) -> Vec3 {
        let x = (2.0 * (self.w * self.x + self.y * self.z)).atan2(1.0 - 2.0 * (self.x * self.x + self.y * self.y));
        let y = (2.0 * (self.w * self.y - self.z * self.x)).asin();
        let z = (2.0 * (self.w * self.z + self.x * self.y)).atan2(1.0 - 2.0 * (self.y * self.y + self.z * self.z));

        Vec3::new(x, y, z)
    }
}