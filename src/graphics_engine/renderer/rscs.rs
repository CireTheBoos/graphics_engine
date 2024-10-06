use ash::vk::{
    BufferCreateInfo, BufferUsageFlags, ComponentMapping, Extent2D, Framebuffer,
    FramebufferCreateInfo, Image, ImageAspectFlags, ImageSubresourceRange, ImageView,
    ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags, SharingMode,
};
use vk_mem::{Alloc, AllocationCreateInfo, Allocator};

use crate::{
    graphics_engine::{allocator::Buffer as CustomBuffer, Device},
    model::{Vertex, MAX_VERTICES},
};

use super::RenderPass;

pub fn create_image_views(device: &Device, images: &Vec<Image>) -> Vec<ImageView> {
    images
        .iter()
        .map(|image| {
            // identity
            let components = ComponentMapping::default();

            let subresource_range = ImageSubresourceRange::default()
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let create_info = ImageViewCreateInfo::default()
                // view restrictions
                .image(*image)
                .view_type(ImageViewType::TYPE_2D)
                .subresource_range(subresource_range)
                // data interpretation
                .format(device.infos.surface_format.format)
                .components(components);

            unsafe { device.create_image_view(&create_info, None) }
                .expect("Failed to create image view.")
        })
        .collect()
}

pub fn create_frame_buffers(
    image_views: &Vec<ImageView>,
    render_pass: &RenderPass,
    extent: &Extent2D,
    device: &Device,
) -> Vec<Framebuffer> {
    image_views
        .iter()
        .map(|view| {
            let attachments = [*view];

            let create_info = FramebufferCreateInfo::default()
                .render_pass(render_pass.render_pass)
                .layers(1)
                .height(extent.height)
                .width(extent.width)
                .attachments(&attachments);

            unsafe { device.create_framebuffer(&create_info, None) }
                .expect("Failed to create framebuffer.")
        })
        .collect()
}

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
