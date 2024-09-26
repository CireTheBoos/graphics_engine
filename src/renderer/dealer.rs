use std::mem::offset_of;

use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, Format, MemoryPropertyFlags, SharingMode,
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};
use gfx_maths::{Vec2, Vec3};
use vk_mem::{Alloc, Allocation, AllocationCreateInfo, Allocator, AllocatorCreateInfo};

use crate::instance::Instance;

use super::Device;

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

pub struct Dealer {
    pub allocator: Allocator,
}

impl Dealer {
    pub fn new(instance: &Instance, device: &Device) -> Dealer {
        let create_info = AllocatorCreateInfo::new(instance, &device, device.infos.physical_device);
        let allocator =
            unsafe { Allocator::new(create_info) }.expect("Failed to create allocator.");
        Dealer { allocator }
    }

    pub fn allocate_vertex_buffer(
        &self,
        device: &Device,
        vertices: &Vec<Vertex>,
    ) -> (Buffer, Allocation) {
        let queue_family_indices = [device.infos.graphics_idx];
        let buffer_info = BufferCreateInfo::default()
            .queue_family_indices(&queue_family_indices)
            .size((size_of::<Vertex>() * vertices.len()) as u64)
            .usage(BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let create_info = AllocationCreateInfo {
            required_flags: MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
            ..Default::default()
        };

        unsafe { self.allocator.create_buffer(&buffer_info, &create_info) }
            .expect("Failed to create vertex buffer.")
    }
}
