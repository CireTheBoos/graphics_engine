

use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, MemoryPropertyFlags, SharingMode,
};
use vk_mem::{Alloc, Allocation, AllocationCreateInfo, Allocator, AllocatorCreateInfo};

use crate::{instance::Instance, model::{Vertex, MAX_VERTICES}};

use super::Device;



// Translates boilerplate resource management code into meaningful fns for renderer
pub struct Dealer {
    pub allocator: Allocator,
    pub vertex_buffer: (Buffer, Allocation),
}

impl Dealer {
    pub fn new(instance: &Instance, device: &Device) -> Dealer {
        let create_info = AllocatorCreateInfo::new(instance, &device, device.infos.physical_device);
        let allocator =
            unsafe { Allocator::new(create_info) }.expect("Failed to create allocator.");
        let vertex_buffer = Dealer::allocate_vertex_buffer(&allocator, device);
        Dealer { allocator, vertex_buffer }
    }

    pub fn destroy(&mut self) {
        unsafe { self.allocator
                .destroy_buffer(self.vertex_buffer.0, &mut self.vertex_buffer.1) };

    }

    fn allocate_vertex_buffer(
        allocator: &Allocator,
        device: &Device,
    ) -> (Buffer, Allocation) {
        let queue_family_indices = [device.infos.graphics_idx];
        let buffer_info = BufferCreateInfo::default()
            .queue_family_indices(&queue_family_indices)
            .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
            .usage(BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let create_info = AllocationCreateInfo {
            required_flags: MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
            ..Default::default()
        };

        unsafe { allocator.create_buffer(&buffer_info, &create_info) }
            .expect("Failed to create vertex buffer.")
    }

    pub fn fill_vertex_buffer(&mut self, vertices: &Vec<Vertex>) {
        let buffer_vertices = unsafe { self.allocator.map_memory(&mut self.vertex_buffer.1) }
            .expect("Failed to map memory.");
        unsafe {
            buffer_vertices.copy_from(
                vertices.as_ptr() as *const u8,
                Vertex::size_of() * vertices.len(),
            )
        };
        unsafe { self.allocator.unmap_memory(&mut self.vertex_buffer.1) };
    }
}
