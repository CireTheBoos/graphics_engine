use std::ops::Deref;

use ash::vk::{
    ComponentMapping, Extent2D, Framebuffer, FramebufferCreateInfo, Image, ImageAspectFlags,
    ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType, RenderPass,
};

use crate::graphics_engine::Device;

pub struct GraphicsFramebuffer {
    framebuffer: Framebuffer,
    swapchain_image_view: ImageView,
}

// Deref : ash::vk::Framebuffer
impl Deref for GraphicsFramebuffer {
    type Target = Framebuffer;
    fn deref(&self) -> &Self::Target {
        &self.framebuffer
    }
}

impl GraphicsFramebuffer {
    pub fn new(
        device: &Device,
        render_pass: &RenderPass,
        images: &Vec<Image>,
    ) -> Vec<GraphicsFramebuffer> {
        images
            .iter()
            .map(|image| {
                let swapchain_image_view = create_swapchain_image_view(device, image);
                let extent = device.infos.capabilities.current_extent;
                let framebuffer =
                    create_framebuffer(device, render_pass, &swapchain_image_view, extent);
                GraphicsFramebuffer {
                    framebuffer,
                    swapchain_image_view,
                }
            })
            .collect()
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
            device.destroy_image_view(self.swapchain_image_view, None);
        }
    }
}

fn create_swapchain_image_view(device: &Device, image: &Image) -> ImageView {
    let components = ComponentMapping::default(); // identity
    let format = device.infos.surface_format.format;

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
        .format(format)
        .components(components);

    unsafe {
        device
            .create_image_view(&create_info, None)
            .expect("Failed to create image view.")
    }
}

fn create_framebuffer(
    device: &Device,
    render_pass: &RenderPass,
    image_view: &ImageView,
    extent: Extent2D,
) -> Framebuffer {
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
