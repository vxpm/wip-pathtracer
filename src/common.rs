use std::ops::Mul;

use crate::Vec3;

/// A ray. Represented by a starting point and a direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    /// The starting point of this ray.
    point: Vec3,
    /// The direction of this ray,
    direction: Vec3,
}

impl Ray {
    /// Creates a new [`Ray`] with the given starting point
    /// and direction.
    ///
    /// # Panics
    /// Panics if `direction` is not normalized.
    #[inline(always)]
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        assert!(direction.is_normalized());
        Self { point, direction }
    }

    /// The starting point of this ray.
    #[inline(always)]
    pub fn point(&self) -> Vec3 {
        self.point
    }

    /// The direction of this ray,
    #[inline(always)]
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Returns the point `ray.point + ray.direction * t`.
    #[inline(always)]
    pub fn point_at_t(&self, t: f32) -> Vec3 {
        self.point + self.direction * t
    }
}

/// A color.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(Vec3);

impl Color {
    /// Creates a new [`Color`] with the given RGB channels.
    ///
    /// # Panics
    /// Panics if any of the channels is not in the \[0..1] range.
    #[inline(always)]
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        assert!(r <= 1.0);
        assert!(g <= 1.0);
        assert!(b <= 1.0);

        Self(Vec3::new(r, g, b))
    }

    #[inline(always)]
    pub fn to_vec3(&self) -> Vec3 {
        self.0
    }
}

impl Mul<Self> for Color {
    type Output = Color;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Color(self.0 * rhs.0)
    }
}
