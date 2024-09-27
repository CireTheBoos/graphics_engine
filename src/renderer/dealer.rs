use ash::vk::{Buffer, BufferCreateInfo, BufferUsageFlags, MemoryPropertyFlags, SharingMode};
use vk_mem::{Alloc, Allocation, AllocationCreateInfo, Allocator, AllocatorCreateInfo};

use crate::{
    instance::Instance,
    model::{Vertex, MAX_VERTICES},
};

use super::Device;

// Translates boilerplate resource management code into meaningful fns for renderer
pub struct Dealer {
    allocator: Allocator,
    pub vertex_buffer: Buffer,
    vertex_allocation: Allocation,
    pub staging_vertex_buffer: Buffer,
    staging_vertex_allocation: Allocation,
}

impl Dealer {
    pub fn new(instance: &Instance, device: &Device) -> Dealer {
        // Allocator
        let create_info = AllocatorCreateInfo::new(instance, &device, device.infos.physical_device);
        let allocator =
            unsafe { Allocator::new(create_info) }.expect("Failed to create allocator.");

        // Allocate buffers
        let (vertex_buffer, vertex_allocation) = allocate_vertex_buffer(&allocator, device);
        let (staging_vertex_buffer, staging_vertex_allocation) = allocate_staging_vertex_buffer(&allocator, device);

        Dealer {
            allocator,
            vertex_buffer,
            vertex_allocation,
            staging_vertex_buffer,
            staging_vertex_allocation,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.allocator
                .destroy_buffer(self.vertex_buffer, &mut self.vertex_allocation);
            self.allocator
                .destroy_buffer(self.staging_vertex_buffer, &mut self.staging_vertex_allocation);
        }
    }

    pub fn update_staging_vertex_buffer(&mut self, vertices: &Vec<Vertex>) {
        unsafe {
            let staging_vertices = self
                .allocator
                .map_memory(&mut self.staging_vertex_allocation)
                .expect("Failed to map memory.");
            staging_vertices.copy_from(
                vertices.as_ptr() as *const u8,
                Vertex::size_of() * vertices.len(),
            );
            self.allocator.unmap_memory(&mut self.staging_vertex_allocation);
        }
    }
}

fn allocate_vertex_buffer(allocator: &Allocator, device: &Device) -> (Buffer, Allocation) {
    let queue_family_indices = [device.infos.graphics_idx, device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::CONCURRENT)
        .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
        .usage(BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::TRANSFER_DST);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
        ..Default::default()
    };

    unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.")
}

fn allocate_staging_vertex_buffer(allocator: &Allocator, device: &Device) -> (Buffer, Allocation) {
    let queue_family_indices = [device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
        .usage(BufferUsageFlags::TRANSFER_SRC);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        ..Default::default()
    };

    unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.")
}
