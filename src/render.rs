use crate::{
    common::Ray,
    light::{Light, LightRay},
    object::Object,
    shape::{Intersect, Intersection},
    Vec3,
};
use float_ord::FloatOrd;
use picture::{prelude::Pixel, view::ImgViewMut};

#[derive(Debug)]
pub struct ViewPlane {
    pub top_left: Vec3,
    pub top_right: Vec3,
    pub bottom_left: Vec3,
    pub bottom_right: Vec3,
}

/// A camera in space that can be used to render what it sees.
pub struct Camera {
    /// The position of this camera in space.
    position: Vec3,
    /// The direction this camera is pointed in. Normalized.
    direction: Vec3,
    /// The field of view of this camera, in radians.
    fov: f32,
    /// The aspect ratio (width / height) of the view plane.
    aspect_ratio: f32,
}

impl Camera {
    /// Creates a new camera with given settings. `fov` is in radians.
    ///
    /// # Panics
    /// Panics if either `direction` is not normalized or `fov` is not
    /// in the `[0, 2pi)` range.
    pub fn new(position: Vec3, direction: Vec3, fov: f32, aspect_ratio: f32) -> Self {
        assert!(0.0 <= fov);
        assert!(fov < 2.0 * std::f32::consts::PI);
        assert!(direction.is_normalized());

        Self {
            position,
            direction,
            fov,
            aspect_ratio,
        }
    }

    /// Returns the direction this camera is looking at.
    /// Alias of [`Camera::z_axis`]. Normalized.
    #[inline(always)]
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Returns the local x axis of this camera. Normalized.
    #[inline(always)]
    pub fn x_axis(&self) -> Vec3 {
        Vec3::Y
            .cross(self.direction)
            .try_normalize()
            .unwrap_or(Vec3::X)
    }

    /// Returns the local y axis of this camera. Normalized.
    #[inline(always)]
    pub fn y_axis(&self) -> Vec3 {
        self.direction
            .cross(self.x_axis())
            .try_normalize()
            .unwrap_or(Vec3::Y)
    }

    /// Returns the local z axis of this camera. Normalized.
    #[inline(always)]
    pub fn z_axis(&self) -> Vec3 {
        self.direction
    }

    /// Returns the 4 extreme points of the view plane
    /// defined by this camera.
    pub fn plane(&self) -> ViewPlane {
        let plane_center = self.position + self.z_axis();
        let half_height = (self.fov / 2.0).tan();
        let half_width = half_height * self.aspect_ratio;

        let top_center = plane_center + self.y_axis() * half_height;
        let top_left = top_center - self.x_axis() * half_width;
        let top_right = top_center + self.x_axis() * half_width;

        let bottom_center = plane_center - self.y_axis() * half_height;
        let bottom_left = bottom_center - self.x_axis() * half_width;
        let bottom_right = bottom_center + self.x_axis() * half_width;

        ViewPlane {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
}

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

pub struct Renderer {
    pub sample_count: u32,
    pub indirect_count: u32,
    pub max_value: f32,
    pub ambient_light: LightRay,
}

impl Renderer {
    pub fn render<I, P>(&self, scene: &Scene, buffer: &mut I)
    where
        I: ImgViewMut<Pixel = P>,
        P: Pixel<Channels = [f32; 3]>,
    {
        let (buffer_width, buffer_height) = buffer.dimensions();
        let plane = scene.camera.plane();

        for y in 0..buffer_height {
            let y_t = (y as f32) / (buffer_height as f32);
            for x in 0..buffer_width {
                let x_t = (x as f32) / (buffer_width as f32);

                let plane_point_top = plane.top_left.lerp(plane.top_right, x_t);
                let plane_point_bottom = plane.bottom_left.lerp(plane.bottom_right, x_t);
                let plane_point = plane_point_top.lerp(plane_point_bottom, y_t);
                let direction = (plane_point - scene.camera.position).normalize();

                let ray = Ray::new(plane_point, direction);
                let mut channels = [0f32; 3];
                for _ in 0..self.sample_count {
                    let light_ray = self.trace_ray(ray, scene, self.indirect_count + 1);
                    let sample = light_ray.to_sample();

                    channels[0] += sample.x;
                    channels[1] += sample.y;
                    channels[2] += sample.z;
                }
                channels[0] /= self.sample_count as f32;
                channels[1] /= self.sample_count as f32;
                channels[2] /= self.sample_count as f32;

                let pixel = buffer.pixel_mut((x, y)).unwrap();
                pixel.channels_mut()[..].copy_from_slice(&channels[..]);
            }
        }

        // dyn linear
        // let mut max = Vec3::ZERO;
        // for channel in buffer.pixels().map(|p| p.channels()) {
        //     let sample = Vec3::from(*channel);
        //     if sample.length_squared() > max.length_squared() {
        //         max = sample;
        //     }
        // }
        // let max_len = max.length();
        // for channel in buffer.pixels_mut().map(|p| p.channels_mut()) {
        //     channel[0] /= max_len;
        //     channel[1] /= max_len;
        //     channel[2] /= max_len;
        // }

        for channel in buffer.pixels_mut().map(|p| p.channels_mut()) {
            channel[0] = (channel[0] / self.max_value).clamp(0.0, 1.0);
            channel[1] = (channel[1] / self.max_value).clamp(0.0, 1.0);
            channel[2] = (channel[2] / self.max_value).clamp(0.0, 1.0);
        }
    }

    pub fn trace_ray(&self, ray: Ray, scene: &Scene, depth: u32) -> LightRay {
        if depth == 0 {
            // ambient color
            return self.ambient_light;
        }

        let closest_hit_object = scene
            .objects
            .iter()
            .filter_map(|obj: &Object| {
                obj.shape
                    .intersection(ray)
                    .map(|intersection| (obj, intersection))
            })
            .min_by_key(|(_, intersection)| FloatOrd(intersection.t));

        let closest_hit_light = scene
            .lights
            .iter()
            .filter_map(|light: &Light| {
                light
                    .shape
                    .intersection(ray)
                    .map(|intersection| (light, intersection))
            })
            .min_by_key(|(_, intersection)| FloatOrd(intersection.t));

        let obj_color = |obj: &Object, intersection: Intersection| {
            let new_ray = obj
                .material
                .scatter(ray, intersection.point, intersection.normal);

            let light_ray = self.trace_ray(new_ray, scene, depth - 1);
            let mat_color = obj.material.color(intersection.point, intersection.normal);

            LightRay {
                color: light_ray.color * mat_color,
                intensity: light_ray.intensity,
            }
        };

        match (closest_hit_light, closest_hit_object) {
            (None, None) => self.ambient_light,
            (None, Some((obj, intersection))) => obj_color(obj, intersection),
            (Some((light, _)), None) => light.light_ray(),
            (Some((light, light_intersection)), Some((obj, obj_intersection))) => {
                if light_intersection.t < obj_intersection.t {
                    light.light_ray()
                } else {
                    obj_color(obj, obj_intersection)
                }
            }
        }
    }
}
