use pathtracer::{
    common::Color,
    light::{Light, LightRay},
    material::Simple,
    object::Object,
    render::{Camera, Renderer, Scene},
    shape::{Plane, Shape, Sphere},
    *,
};
use picture::{
    formats::png::PngEncoder,
    prelude::{ImgBuf, RGB, RGB8},
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{io::Write, sync::Arc, time::Duration};

fn main() {
    let start = std::time::Instant::now();
    // renders exactly one frame and saves as 0.png
    render_anim(Duration::from_secs_f32(1.0 / 15.0), 15f32);
    println!("{:?}", start.elapsed());
}

fn render_anim(duration: Duration, fps: f32) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .unwrap();

    let duration_secs = duration.as_secs_f32();
    let frame_time = 1.0 / fps;
    let frame_count = (duration_secs * fps).ceil() as u32;

    // materials
    let material_red = Arc::new(Simple {
        color: Color::new(1.0, 0.0, 0.0),
        diffuse: 1.0,
        fuzzyness: 0.0,
    });

    let material_green = Arc::new(Simple {
        color: Color::new(0.0, 1.0, 0.0),
        diffuse: 1.0,
        fuzzyness: 0.0,
    });

    let material_blue = Arc::new(Simple {
        color: Color::new(0.2, 0.2, 1.0),
        diffuse: 0.3,
        fuzzyness: 0.05,
    });

    let material_white = Arc::new(Simple {
        color: Color::new(1.0, 1.0, 1.0),
        diffuse: 1.0,
        fuzzyness: 0.0,
    });

    let material_mirror = Arc::new(Simple {
        color: Color::new(1.0, 1.0, 1.0),
        diffuse: 0.0,
        fuzzyness: 0.0,
    });

    let material_mirror_fuzzy = Arc::new(Simple {
        color: Color::new(0.5, 0.8, 0.8),
        diffuse: 0.2,
        fuzzyness: 0.08,
    });

    let material_pink = Arc::new(Simple {
        color: Color::new(0.94, 0.3, 0.85),
        diffuse: 0.5,
        fuzzyness: 0.02,
    });

    let material_black = Arc::new(Simple {
        color: Color::new(0.05, 0.05, 0.05),
        diffuse: 1.0,
        fuzzyness: 0.0,
    });

    (0..frame_count).into_par_iter().for_each(|frame| {
        let time = frame as f32 * frame_time;

        // camera
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::new(0.0, 0.0, 1.0).normalize(),
            75f32.to_radians(),
            1.0,
        );

        // box
        let floor = Object {
            shape: Shape::from(Plane::new(Vec3::new(0.0, -5.0, 0.0), Vec3::Y)),
            material: material_blue.clone(),
        };

        let wall_left = Object {
            shape: Shape::from(Plane::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X)),
            material: material_red.clone(),
        };

        let wall_right = Object {
            shape: Shape::from(Plane::new(Vec3::new(5.0, 0.0, 0.0), -Vec3::X)),
            material: material_green.clone(),
        };

        let wall_front = Object {
            shape: Shape::from(Plane::new(Vec3::new(0.0, 0.0, 5.0), -Vec3::Z)),
            material: material_white.clone(),
        };

        let ceil = Object {
            shape: Shape::from(Plane::new(Vec3::new(0.0, 5.0, 0.0), -Vec3::Y)),
            material: material_mirror_fuzzy.clone(),
        };

        // floating spheres
        let sphere_mirror = Object {
            shape: Shape::from(Sphere {
                center: Vec3::new(
                    3.0 * (2.0 * std::f32::consts::PI * time / duration_secs
                        + 3.0 * std::f32::consts::PI / 2.0)
                        .sin(),
                    -3.0,
                    -3.0,
                ),
                radius: 1.0,
            }),
            material: material_mirror.clone(),
        };

        let sphere_pink = Object {
            shape: Shape::from(Sphere {
                center: Vec3::new(
                    3.0,
                    3.0 * (2.0 * std::f32::consts::PI * time / duration_secs
                        + 3.0 * std::f32::consts::PI / 2.0)
                        .sin(),
                    3.0,
                ),
                radius: 2.0,
            }),
            material: material_pink.clone(),
        };

        let sphere_black_a = Object {
            shape: Shape::from(Sphere {
                center: Vec3::new(
                    2.0 * (2.0 * std::f32::consts::PI * time / duration_secs).sin(),
                    3.5,
                    2.0 * (2.0 * std::f32::consts::PI * time / duration_secs).cos(),
                ),
                radius: 0.5,
            }),
            material: material_black.clone(),
        };

        let sphere_black_b = Object {
            shape: Shape::from(Sphere {
                center: Vec3::new(
                    -2.0 * (2.0 * std::f32::consts::PI * time / duration_secs).sin(),
                    -3.5,
                    -2.0 * (2.0 * std::f32::consts::PI * time / duration_secs).cos(),
                ),
                radius: 0.5,
            }),
            material: material_black.clone(),
        };

        // light sphere
        let light = Light::new(
            Shape::from(Sphere {
                center: Vec3::new(
                    0.0,
                    1.0 * (2.0 * std::f32::consts::PI * time / duration_secs).sin(),
                    0.0,
                ),
                radius: 2.0,
            }),
            Color::new(1.0, 1.0, 1.0),
            2048.0,
        );

        let scene = Scene {
            camera,
            objects: vec![
                floor,
                wall_left,
                wall_right,
                wall_front,
                ceil,
                sphere_mirror,
                sphere_pink,
                sphere_black_a,
                sphere_black_b,
            ],
            lights: vec![light],
        };

        let mut buffer = ImgBuf::<RGB<f32>, Vec<_>>::new(512, 512);
        Renderer {
            sample_count: 128,
            indirect_count: 4,
            max_value: 1024.0,
            ambient_light: LightRay {
                color: Color::new(0.0, 0.0, 0.0),
                intensity: 0.0,
            },
        }
        .render(&scene, &mut buffer);

        let result = buffer.map_vec(|x| {
            RGB8::new(
                (x.r.sqrt() * 255.0) as u8,
                (x.g.sqrt() * 255.0) as u8,
                (x.b.sqrt() * 255.0) as u8,
            )
        });

        let encoded = PngEncoder::default().encode(result).unwrap();
        let mut f = std::fs::File::create(format!("{frame}.png")).unwrap();
        f.write_all(&encoded[..]).unwrap();
    });
}
