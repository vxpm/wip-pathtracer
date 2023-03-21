use crate::Vec3;
use crate::{common::Color, shape::Shape};

/// A light in a scene.
pub struct Light {
    /// The shape of this light.
    pub shape: Shape,
    /// The color of this light.
    pub color: Color,
    /// The intensity of this light.
    pub intensity: f32,
}

impl Light {
    pub fn new(shape: Shape, color: Color, intensity: f32) -> Self {
        Self {
            shape,
            color,
            intensity,
        }
    }

    #[inline(always)]
    pub fn light_ray(&self) -> LightRay {
        LightRay {
            color: self.color,
            intensity: self.intensity,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightRay {
    pub color: Color,
    pub intensity: f32,
}

impl LightRay {
    #[inline(always)]
    pub fn to_sample(&self) -> Vec3 {
        self.color.to_vec3() * self.intensity
    }
}
