use std::{mem::offset_of, time::Instant};

use ash::vk::{Format, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate};
use gfx_maths::{Vec2, Vec3};

pub const MAX_VERTICES: u64 = 12;

pub struct Model {
    now: Instant,
    pub vertices: Vec<Vertex>,
    switch: u32,
}

impl Model {
    pub fn new() -> Model {
        Model {
            now: Instant::now(),
            vertices: vertices(),
            switch: 0
        }
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    // step every second
    pub fn step_if_enough_time(&mut self) {
        if self.now.elapsed().as_millis() >= 1000 {
            if self.switch == 0 {
                self.switch = 1;
                self.vertices = vertices_2();
            }
            else {
                self.switch = 0;
                self.vertices = vertices();
            }
            self.now = Instant::now();
        }
    }
}

#[repr(C)]
pub struct Vertex {
    pos: Vec2,
    color: Vec3,
}

impl Vertex {
    pub fn binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(VertexInputRate::VERTEX)
    }
    pub fn attribute_description() -> Vec<VertexInputAttributeDescription> {
        let pos_description = VertexInputAttributeDescription::default()
            .binding(0)
            .format(Format::R32G32_SFLOAT)
            .location(0)
            .offset(offset_of!(Vertex, pos) as u32);
        let color_description = VertexInputAttributeDescription::default()
            .binding(0)
            .format(Format::R32G32B32_SFLOAT)
            .location(1)
            .offset(offset_of!(Vertex, color) as u32);
        vec![pos_description, color_description]
    }
    pub fn size_of() -> usize {
        size_of::<Vertex>()
    }
}

pub fn vertices() -> Vec<Vertex> {
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