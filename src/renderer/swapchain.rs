use std::ops::Deref;

use ash::vk::{
    ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Image, ImageAspectFlags,
    ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType,
    SharingMode, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
};

use super::device::RendererDevice;

pub struct RendererSwapchain {
    swapchain: SwapchainKHR,
    images: Vec<Image>,
}

// Deref to ash::vk::SwapchainKHR
impl Deref for RendererSwapchain {
    type Target = SwapchainKHR;
    fn deref(&self) -> &Self::Target {
        &self.swapchain
    }
}

impl RendererSwapchain {
    pub fn new(device: &RendererDevice, surface: &SurfaceKHR) -> RendererSwapchain {
        // device data
        let infos = &device.infos;

        // SPECIFY : minimum image count (triple buffering if possible)
        let min_image_count =
            if infos.capabilities.min_image_count == infos.capabilities.max_image_count {
                infos.capabilities.min_image_count
            } else {
                infos.capabilities.min_image_count + 1
            };

        // SPECIFY : behaviour, image format
        let mut create_info = SwapchainCreateInfoKHR::default()
            // Behaviour
            .min_image_count(min_image_count)
            .present_mode(infos.present_mode)
            .pre_transform(infos.capabilities.current_transform)
            // Image format (all compatible with surface)
            .surface(*surface)
            .image_format(infos.surface_format.format)
            .image_color_space(infos.surface_format.color_space)
            .image_extent(infos.capabilities.current_extent)
            // interactions with other windows (I don't use)
            .clipped(true) // skip pxls culled by another window
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            // others (I don't use)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .old_swapchain(SwapchainKHR::null());

        // SPECIFY : queues sharing mode
        let queues = [infos.graphics_idx, infos.present_idx];
        if infos.graphics_idx != infos.present_idx {
            create_info = create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&queues);
        } else {
            create_info = create_info.image_sharing_mode(SharingMode::EXCLUSIVE);
        }

        // CREATE : swapchain
        let swapchain = unsafe { device.swapchain_khr().create_swapchain(&create_info, None) }
            .expect("Failed to create swapchain.");

        // CREATE : images
        let images = unsafe { device.swapchain_khr().get_swapchain_images(swapchain) }
            .expect("Failed to extract images.");

        RendererSwapchain { swapchain, images }
    }

    pub fn get_image_views(&self, device: &RendererDevice) -> Vec<ImageView> {
        self.images
            .iter()
            .map(|image| {
                let components = ComponentMapping::default()
                    .a(ComponentSwizzle::IDENTITY)
                    .r(ComponentSwizzle::IDENTITY)
                    .g(ComponentSwizzle::IDENTITY)
                    .b(ComponentSwizzle::IDENTITY);

                let subresource_range = ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);

                let create_info = ImageViewCreateInfo::default()
                    .view_type(ImageViewType::TYPE_2D)
                    .image(*image)
                    .format(device.infos.surface_format.format)
                    .components(components)
                    .subresource_range(subresource_range);

                unsafe { device.create_image_view(&create_info, None) }
                    .expect("Failed to create image view.")
            })
            .collect()
    }
}
