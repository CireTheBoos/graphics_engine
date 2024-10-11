mod swapchain_images;
mod vertices;

use crate::graphics_engine::{device::CustomBuffer, Device};
use ash::vk::BufferCreateInfo;
use vk_mem::{Alloc, AllocationCreateInfo};

pub use vertices::{
    allocate_indices, allocate_staging_indices, allocate_staging_vertices, allocate_vertices,
};

pub use swapchain_images::create_swapchain_image_views;

fn create_buffer(
    device: &Device,
    buffer_info: &BufferCreateInfo,
    create_info: &AllocationCreateInfo,
) -> CustomBuffer {
    let (buffer, allocation) = unsafe {
        device
            .allocator()
            .create_buffer(buffer_info, create_info)
            .expect("Failed to create vertex buffer.")
    };
    CustomBuffer { buffer, allocation }
}
