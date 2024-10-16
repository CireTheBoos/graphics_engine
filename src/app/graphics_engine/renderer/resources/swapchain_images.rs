use ash::vk::{
    ComponentMapping, Image, ImageAspectFlags, ImageSubresourceRange, ImageView,
    ImageViewCreateInfo, ImageViewType,
};

use crate::app::graphics_engine::Device;

pub fn create_swapchain_image_views(
    device: &Device,
    swapchain_images: &Vec<Image>,
) -> Vec<ImageView> {
    swapchain_images
        .iter()
        .map(|image| create_swapchain_image_view(device, image))
        .collect()
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
