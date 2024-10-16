mod camera;
pub mod object;
pub mod space;

pub use camera::Camera;
use glam::{Quat, Vec3};
use object::{Cube, Octahedron};
use space::Coord;
use std::time::Instant;

use super::graphics_engine::ToMesh;

// Handle vertices based on time
pub struct Model {
    camera: Camera,
    // Objects
    octahedrons: Vec<Octahedron>,
    cubes: Vec<Cube>,
    // Stepping
    last_step: Instant,
}

impl Model {
    pub fn new() -> Model {
        let camera = Camera::new(2. * Vec3::ONE, Vec3::ZERO);
        let octahedron_1 = Octahedron::new_unoriented(Coord::new(0.5, 0.5, 0.5), 0.5);
        let cube_1 = Cube::new_unoriented(Coord::new(-0.5, -0.5, -0.5), 0.25);
        Model {
            camera,
            octahedrons: vec![octahedron_1],
            cubes: vec![cube_1],
            last_step: Instant::now(),
        }
    }

    pub fn step_if_enough_time(&mut self) {
        let elapsed = self.last_step.elapsed().as_millis();
        if elapsed >= 20 {
            self.octahedrons[0]
                .orientation
                .rotate(Quat::from_rotation_x(0.1));
            self.cubes[0].orientation.rotate(Quat::from_rotation_x(0.1));
            self.last_step = Instant::now();
        }
    }

    pub fn objects_to_draw(&self) -> Vec<&dyn ToMesh> {
        let mut objects: Vec<&dyn ToMesh> =
            Vec::with_capacity(self.cubes.len() + self.octahedrons.len());
        for octahedron in &self.octahedrons {
            objects.push(octahedron);
        }
        for cube in &self.cubes {
            objects.push(cube);
        }
        objects
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}
