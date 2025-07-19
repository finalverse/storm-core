// File: crates/storm-math/src/spatial.rs
// Spatial mathematics utilities

use crate::Vec3;

/// Bounding box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn contains(self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
            point.y >= self.min.y && point.y <= self.max.y &&
            point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(self) -> Vec3 {
        self.max - self.min
    }
}

/// Sphere
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn contains(self, point: Vec3) -> bool {
        self.center.distance(point) <= self.radius
    }
}

/// Spatial utility functions
pub fn distance(a: Vec3, b: Vec3) -> f32 {
    a.distance(b)
}

pub fn point_in_sphere(point: Vec3, center: Vec3, radius: f32) -> bool {
    distance(point, center) <= radius
}

pub fn point_in_aabb(point: Vec3, min: Vec3, max: Vec3) -> bool {
    point.x >= min.x && point.x <= max.x &&
        point.y >= min.y && point.y <= max.y &&
        point.z >= min.z && point.z <= max.z
}

pub fn lerp_transform(a: crate::Transform, b: crate::Transform, t: f32) -> crate::Transform {
    crate::Transform {
        position: a.position.lerp(b.position, t),
        rotation: a.rotation.slerp(b.rotation, t),
        scale: a.scale.lerp(b.scale, t),
    }
}