use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::element::Container;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[repr(C)]
pub struct Vector(pub f32, pub f32);

impl Vector {
    pub const ZERO: Self = Self(0.0, 0.0);

    pub fn diagonal(v: f32) -> Self {
        Self(v, v)
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self(x, y)
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs(), self.1.abs())
    }

    pub fn max(&self, v: f32) -> Self {
        Self(self.0.max(v), self.1.max(v))
    }

    pub fn min(&self, v: f32) -> Self {
        Self(self.0.min(v), self.1.min(v))
    }

    /// Rotates the vector around a given point by a specified angle in radians.
    pub fn rotate_around_point(&self, point: &Self, angle: f32) -> Self {
        let (cx, cy) = (point.0, point.1);
        let (x, y) = (self.0, self.1);

        let translated_x = x - cx;
        let translated_y = y - cy;

        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let rotated_x = translated_x * cos_angle - translated_y * sin_angle;
        let rotated_y = translated_x * sin_angle + translated_y * cos_angle;

        let result_x = rotated_x + cx;
        let result_y = rotated_y + cy;

        Self(result_x, result_y)
    }

    pub fn rotate_around_origin(&self, angle: f32) -> Self {
        let (x, y) = (self.0, self.1);
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let rotated_x = x * cos_angle - y * sin_angle;
        let rotated_y = x * sin_angle + y * cos_angle;

        Self(rotated_x, rotated_y)
    }

    pub fn container_colision(&self, c: &Container) -> Option<Vector> {
        if c.rotation == 0.0 {
            return self
                .rectangle_colision(&c.pos, &c.size)
                .then(|| *self - c.pos);
        }

        let rot = self.rotate_around_point(&c.pos, -c.rotation);

        rot.rectangle_colision(&c.pos, &c.size).then(|| rot - c.pos)
    }

    pub fn container_colision_with_pos(&self, c: &Container) -> (bool, Vector) {
        if c.rotation == 0.0 {
            return (self.rectangle_colision(&c.pos, &c.size), *self - c.pos);
        }

        let rot = self.rotate_around_point(&c.pos, -c.rotation);

        (rot.rectangle_colision(&c.pos, &c.size), rot - c.pos)
    }

    pub fn rectangle_colision(&self, pos: &Self, size: &Self) -> bool {
        let (w, h) = (size.0 / 2.0, size.1 / 2.0);
        self.0 >= pos.0 - w && self.0 <= pos.0 + w && self.1 >= pos.1 - h && self.1 <= pos.1 + h
    }

    pub fn relative_pos(&self, pos: &Self, rot: f32) -> Self {
        if rot == 0.0 {
            return *self - *pos;
        }

        self.rotate_around_point(pos, rot) - *pos
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0 && self.1 == 0.0
    }
}

impl From<(f32, f32)> for Vector {
    fn from(value: (f32, f32)) -> Self {
        Self(value.0, value.1)
    }
}

impl From<Vector> for (f32, f32) {
    fn from(value: Vector) -> Self {
        (value.0, value.1)
    }
}

impl From<[f32; 2]> for Vector {
    fn from(value: [f32; 2]) -> Self {
        Self(value[0], value[1])
    }
}

impl From<Vector> for [f32; 2] {
    fn from(value: Vector) -> Self {
        [value.0, value.1]
    }
}

impl From<(u32, u32)> for Vector {
    fn from(value: (u32, u32)) -> Self {
        Self(value.0 as f32, value.1 as f32)
    }
}

impl Add<Vector> for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Add<f32> for Vector {
    type Output = Vector;

    fn add(self, rhs: f32) -> Self::Output {
        Self(self.0 + rhs, self.1 + rhs)
    }
}

impl Sub<Vector> for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Sub<f32> for Vector {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self(self.0 - rhs, self.1 - rhs)
    }
}

impl Mul<Vector> for Vector {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl Div<Vector> for Vector {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs)
    }
}

impl Rem<Vector> for Vector {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0, self.1 % rhs.1)
    }
}

impl Rem<f32> for Vector {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self::Output {
        Self(self.0 % rhs, self.1 % rhs)
    }
}
