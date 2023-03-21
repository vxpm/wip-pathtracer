use crate::common::Ray;
use crate::{Vec3, EPSILON};
use enum_dispatch::enum_dispatch;

/// An intersection of a [`Ray`] with some sort of [`Shape`].
pub struct Intersection {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
}

/// Trait for things in space that can intersect with a ray.
#[enum_dispatch]
pub trait Intersect {
    fn intersection(&self, ray: Ray) -> Option<Intersection>;
}

/// A sphere shape.
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    #[inline(always)]
    pub fn normal(&self, point: Vec3) -> Vec3 {
        (point - self.center).try_normalize().unwrap_or(Vec3::Z)
    }
}

impl Intersect for Sphere {
    fn intersection(&self, ray: Ray) -> Option<Intersection> {
        // all we have to do is solve the following equation:
        // (P + d * t - C)² = r²
        // which is the same as stating that the distance of the
        // intersection to the center of the sphere should be
        // equal to it's radius.
        //
        // after development of the aforementioned equation,
        // you end up with the following 2nd degree equation:
        // t² + 2(P - C).d * t + (P - C)² - r² = 0
        // where:
        // a = 1
        // b = 2(P - C).d
        // c = (P - C)² - r²

        let p_minus_c = ray.point() - self.center;
        let b_halved = p_minus_c.dot(ray.direction());
        let c = p_minus_c.length_squared() - self.radius * self.radius;

        // this is delta / 4
        let delta_reduced = b_halved * b_halved - c;
        if delta_reduced < 0.0 {
            return None;
        }

        let delta_sqrt_halved = delta_reduced.sqrt();

        let t1 = -b_halved + delta_sqrt_halved;
        let t2 = -b_halved - delta_sqrt_halved;

        let (min, max) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

        let valid_t = if min >= EPSILON {
            Some(min)
        } else if max >= EPSILON {
            Some(max)
        } else {
            None
        };

        valid_t.map(|t| {
            let point = ray.point_at_t(t);
            Intersection {
                point,
                normal: self.normal(point),
                t,
            }
        })
    }
}

/// A plane shape.
pub struct Plane {
    point: Vec3,
    normal: Vec3,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        assert!(normal.is_normalized());
        Self { point, normal }
    }
}

impl Intersect for Plane {
    fn intersection(&self, ray: Ray) -> Option<Intersection> {
        let dir_dot_normal = ray.direction().dot(self.normal);

        if dir_dot_normal.abs() > 0.0 {
            let plane_point_minus_ray_point = self.point - ray.point();
            let t = plane_point_minus_ray_point.dot(self.normal) / dir_dot_normal;

            if t > EPSILON {
                return Some(Intersection {
                    point: ray.point_at_t(t),
                    normal: self.normal,
                    t,
                });
            }
        }

        None
    }
}

/// A shape in space - just something that can be tested for intersection
/// with a ray.
#[enum_dispatch(Intersect)]
pub enum Shape {
    Sphere(Sphere),
    Plane(Plane),
}
