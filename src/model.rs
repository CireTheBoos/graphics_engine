mod vertex;

use gfx_maths::{Vec2, Vec3};
use std::time::Instant;
pub use vertex::Vertex;

pub const MAX_VERTICES: u64 = 12;

// Handle vertices based on time
pub struct Model {
    now: Instant,
    pub vertices: Vec<Vertex>,
    switch: u32,
}

impl Model {
    pub fn new() -> Model {
        Model {
            now: Instant::now(),
            vertices: vertices_1(),
            switch: 0,
        }
    }

    pub fn step_if_enough_time(&mut self) {
        if self.now.elapsed().as_millis() >= 1000 {
            if self.switch == 0 {
                self.switch = 1;
                self.vertices = vertices_2();
            } else {
                self.switch = 0;
                self.vertices = vertices_1();
            }
            self.now = Instant::now();
        }
    }
}

pub fn vertices_1() -> Vec<Vertex> {
    let vertex_1 = Vertex {
        pos: Vec2::new(-0.25, -0.75),
        color: Vec3::new(0., 0., 1.),
    };
    let vertex_2 = Vertex {
        pos: Vec2::new(-0.25, -0.25),
        color: Vec3::new(0., 1., 0.),
    };
    let vertex_3 = Vertex {
        pos: Vec2::new(-0.75, -0.25),
        color: Vec3::new(1., 0., 0.),
    };
    vec![vertex_1, vertex_2, vertex_3]
}

pub fn vertices_2() -> Vec<Vertex> {
    let vertex_1 = Vertex {
        pos: Vec2::new(0.25, 0.75),
        color: Vec3::new(0., 0., 1.),
    };
    let vertex_2 = Vertex {
        pos: Vec2::new(0.25, 0.25),
        color: Vec3::new(0., 1., 0.),
    };
    let vertex_3 = Vertex {
        pos: Vec2::new(0.75, 0.25),
        color: Vec3::new(1., 0., 0.),
    };
    vec![vertex_1, vertex_2, vertex_3]
}
