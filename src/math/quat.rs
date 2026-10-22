use std::ops::{Mul, MulAssign};

use super::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub const fn new(
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let axis = axis.normalized();

        if axis.length_squared() == 0.0 {
            return Self::IDENTITY;
        }

        let half_angle = angle * 0.5;
        let sin = half_angle.sin();

        Self {
            x: axis.x * sin,
            y: axis.y * sin,
            z: axis.z * sin,
            w: half_angle.cos(),
        }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x
            + self.y * self.y
            + self.z * self.z
            + self.w * self.w
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();

        if length == 0.0 {
            Self::IDENTITY
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
                z: self.z / length,
                w: self.w / length,
            }
        }
    }

    pub fn conjugate(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn rotate_vec3(self, vector: Vec3) -> Vec3 {
        let q = self.normalized();

        let vector_quat = Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
            w: 0.0,
        };

        let rotated =
            q * vector_quat * q.conjugate();

        Vec3::new(
            rotated.x,
            rotated.y,
            rotated.z,
        )
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Quat {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            w:
                self.w * other.w
                - self.x * other.x
                - self.y * other.y
                - self.z * other.z,

            x:
                self.w * other.x
                + self.x * other.w
                + self.y * other.z
                - self.z * other.y,

            y:
                self.w * other.y
                - self.x * other.z
                + self.y * other.w
                + self.z * other.x,

            z:
                self.w * other.z
                + self.x * other.y
                - self.y * other.x
                + self.z * other.w,
        }
    }
}

impl MulAssign for Quat {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}