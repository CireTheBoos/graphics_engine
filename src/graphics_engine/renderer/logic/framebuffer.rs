use ash::vk::{Extent2D, Framebuffer, FramebufferCreateInfo, ImageView, RenderPass};

use crate::graphics_engine::Device;

pub fn create_framebuffers(
    device: &Device,
    render_pass: &RenderPass,
    swapchain_image_views: &Vec<ImageView>,
) -> Vec<Framebuffer> {
    let extent = device.infos.capabilities.current_extent;
    swapchain_image_views
        .iter()
        .map(|swapchain_image_view| {
            create_framebuffer(device, render_pass, swapchain_image_view, extent)
        })
        .collect()
}

fn create_framebuffer(
    device: &Device,
    render_pass: &RenderPass,
    image_view: &ImageView,
    extent: Extent2D,
) -> ash::vk::Framebuffer {
    let attachments = [*image_view];

    let create_info = FramebufferCreateInfo::default()
        .render_pass(*render_pass)
        .layers(1)
        .height(extent.height)
        .width(extent.width)
        .attachments(&attachments);

    unsafe {
        device
            .create_framebuffer(&create_info, None)
            .expect("Failed to create framebuffer.")
    }
}
