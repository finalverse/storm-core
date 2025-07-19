// File: crates/storm-math/src/vector.rs
// Vector mathematics

/// Additional vector utilities
pub mod utils {
    use crate::Vec3;

    pub fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
        Vec3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
    }

    pub fn slerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
        // Spherical linear interpolation
        let dot = a.dot(b);
        if dot > 0.9995 {
            return lerp(a, b, t);
        }

        let theta = dot.acos();
        let sin_theta = theta.sin();

        let a_factor = ((1.0 - t) * theta).sin() / sin_theta;
        let b_factor = (t * theta).sin() / sin_theta;

        a * a_factor + b * b_factor
    }
}