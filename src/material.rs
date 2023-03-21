use rand::{
    rngs::{SmallRng, ThreadRng},
    Rng, SeedableRng,
};

use crate::{
    common::{Color, Ray},
    Vec3,
};

/// A material. Dictates how light scatters off of [objects](Object) made
/// of it.
pub trait Material {
    /// Scatters a ray from the given point and normal.
    fn scatter(&self, ray: Ray, point: Vec3, normal: Vec3) -> Ray;
    fn color(&self, point: Vec3, normal: Vec3) -> Color;
}

pub struct Simple {
    pub color: Color,
    pub diffuse: f32,
    pub fuzzyness: f32,
}

impl Material for Simple {
    fn scatter(&self, ray: Ray, point: Vec3, normal: Vec3) -> Ray {
        let mut rng = SmallRng::from_rng(rand::thread_rng()).unwrap();
        let random_vec_unit_sphere = |rng: &mut SmallRng| loop {
            let v = Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
            );
            if v.length_squared() < 1.0 {
                return v.normalize();
            }
        };

        if self.diffuse > rng.gen_range(0.0..1.0) {
            // diffuse
            let center = point + normal;
            let random = random_vec_unit_sphere(&mut rng);

            let dir = ((center + random) - point).normalize();

            Ray::new(point, dir)
        } else {
            // reflection
            let factor = 2.0 * ray.direction().dot(normal);
            let dir = ray.direction() - factor * normal;
            let fuzz = self.fuzzyness * random_vec_unit_sphere(&mut rng);

            Ray::new(point, (dir + fuzz).normalize())
        }
    }

    #[inline(always)]
    fn color(&self, _: Vec3, _: Vec3) -> Color {
        self.color
    }
}
