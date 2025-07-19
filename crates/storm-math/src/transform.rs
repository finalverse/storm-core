// File: crates/storm-math/src/transform.rs
// Transform utilities

use crate::{Transform, Vec3};

impl Transform {
    /// Apply transform to a point
    pub fn apply(self, point: Vec3) -> Vec3 {
        // Scale, rotate, then translate
        let scaled = Vec3::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
            point.z * self.scale.z,
        );

        // Apply rotation (simplified)
        let rotated = scaled; // TODO: proper quaternion rotation

        rotated + self.position
    }

    /// Combine two transforms
    pub fn combine(self, other: Transform) -> Transform {
        Transform {
            position: self.apply(other.position),
            rotation: self.rotation, // TODO: proper quaternion multiplication
            scale: Vec3::new(
                self.scale.x * other.scale.x,
                self.scale.y * other.scale.y,
                self.scale.z * other.scale.z,
            ),
        }
    }
}