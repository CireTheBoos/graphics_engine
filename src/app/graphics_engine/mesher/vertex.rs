use std::mem::offset_of;

use ash::vk::{
    Format, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};
use glam::Vec3;

#[repr(C)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Vec3,
}

impl Vertex {
    pub fn new(pos: Vec3, color: Vec3) -> Vertex {
        Vertex { pos, color }
    }
    pub fn size_of() -> usize {
        size_of::<Vertex>()
    }
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
}
