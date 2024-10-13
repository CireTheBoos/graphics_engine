use ash::vk::{BufferCreateInfo, BufferUsageFlags, MemoryPropertyFlags, SharingMode};
use vk_mem::AllocationCreateInfo;

use crate::app::graphics_engine::{
    device::CustomBuffer,
    mesher::{Vertex, MAX_INDICES, MAX_VERTICES},
    Device,
};

pub fn allocate_vertices(device: &Device) -> CustomBuffer {
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

    super::create_buffer(device, &buffer_info, &create_info)
}

pub fn allocate_staging_vertices(device: &Device) -> CustomBuffer {
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

    super::create_buffer(device, &buffer_info, &create_info)
}

pub fn allocate_indices(device: &Device) -> CustomBuffer {
    let queue_family_indices = [device.infos.graphics_idx, device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::CONCURRENT)
        .size(size_of::<u32>() as u64 * MAX_INDICES)
        .usage(BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
        ..Default::default()
    };

    super::create_buffer(device, &buffer_info, &create_info)
}

pub fn allocate_staging_indices(device: &Device) -> CustomBuffer {
    let queue_family_indices = [device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .size(size_of::<u32>() as u64 * MAX_INDICES)
        .usage(BufferUsageFlags::TRANSFER_SRC);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        ..Default::default()
    };

    super::create_buffer(device, &buffer_info, &create_info)
}
