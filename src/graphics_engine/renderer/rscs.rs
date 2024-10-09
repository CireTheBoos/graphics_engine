use ash::vk::{BufferCreateInfo, BufferUsageFlags, MemoryPropertyFlags, SharingMode};
use vk_mem::{Alloc, AllocationCreateInfo, Allocator};

use crate::{
    graphics_engine::{device::CustomBuffer, Device},
    model::{Vertex, MAX_VERTICES},
};

pub fn allocate_vertex_buffer(allocator: &Allocator, device: &Device) -> CustomBuffer {
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

    let (buffer, allocation) = unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.");

    CustomBuffer { buffer, allocation }
}

pub fn allocate_staging_vertex_buffer(allocator: &Allocator, device: &Device) -> CustomBuffer {
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

    let (buffer, allocation) = unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.");

    CustomBuffer { buffer, allocation }
}
