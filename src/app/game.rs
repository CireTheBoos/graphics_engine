mod camera;
pub mod objects;
mod space;

pub use camera::Camera;
use glam::Vec3;
use objects::Square;
use space::Coord;
use std::time::Instant;

pub const MAX_VERTICES: u64 = 12;
pub const MAX_INDICES: u64 = 32;

// Handle vertices based on time
pub struct Model {
    pub squares: Vec<Square>,
    pub camera: Camera,
    last_step: Instant,
    switch: u32,
}

impl Model {
    pub fn new() -> Model {
        let camera = Camera::new(2. * Vec3::ONE, Vec3::ZERO);
        let square = Square::new(Coord::new(0.5, 0.5, 1.), 0.5);
        Model {
            squares: vec![square],
            camera,
            last_step: Instant::now(),
            switch: 0,
        }
    }

    pub fn step_if_enough_time(&mut self) {
        if self.last_step.elapsed().as_millis() >= 1000 {
            if self.switch == 0 {
                self.switch = 1;
                self.squares[0].position = Coord::new(-0.5, -0.5, 1.);
                self.squares[0].size = 0.25;
            } else {
                self.switch = 0;
                self.squares[0].position = Coord::new(0.5, 0.5, 1.);
                self.squares[0].size = 0.5;
            }
            self.last_step = Instant::now();
        }
    }
}
