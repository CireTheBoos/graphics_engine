mod vertex;

use gfx_maths::{Vec2, Vec3};
use std::time::Instant;
pub use vertex::Vertex;

pub const MAX_VERTICES: u64 = 12;
pub const MAX_INDICES: u64 = 32;

// Handle vertices based on time
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    last_step: Instant,
    switch: u32,
}

impl Model {
    pub fn new() -> Model {
        Model {
            vertices: vertices_1(),
            indices: vec![0, 1, 2, 0, 2, 3],
            last_step: Instant::now(),
            switch: 0,
        }
    }

    pub fn step_if_enough_time(&mut self) {
        if self.last_step.elapsed().as_millis() >= 1000 {
            if self.switch == 0 {
                self.switch = 1;
                self.vertices = vertices_2();
            } else {
                self.switch = 0;
                self.vertices = vertices_1();
            }
            self.last_step = Instant::now();
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
    let vertex_4 = Vertex {
        pos: Vec2::new(-0.75, -0.75),
        color: Vec3::new(1., 0., 0.),
    };
    vec![vertex_1, vertex_2, vertex_3, vertex_4]
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
    let vertex_4 = Vertex {
        pos: Vec2::new(0.75, 0.75),
        color: Vec3::new(1., 0., 0.),
    };
    vec![vertex_1, vertex_2, vertex_3, vertex_4]
}
