use crate::app::graphics_engine::device::Device;
use ash::vk::{
    CompositeAlphaFlagsKHR, Image, ImageUsageFlags, SharingMode, SurfaceKHR,
    SwapchainCreateInfoKHR, SwapchainKHR,
};
use std::ops::Deref;

// Custom swapchain for presenter.
// - at least 3 images
// - hold swapchain images
pub struct Swapchain {
    swapchain: SwapchainKHR,
    pub images: Vec<Image>,
}

// Deref : ash::vk::SwapchainKHR
impl Deref for Swapchain {
    type Target = SwapchainKHR;
    fn deref(&self) -> &Self::Target {
        &self.swapchain
    }
}

impl Swapchain {
    pub fn new(device: &Device, surface: &SurfaceKHR) -> Swapchain {
        let infos = &device.infos;

        // SPECIFY : minimum image count (triple buffering if possible)
        let min_image_count =
            if infos.capabilities.min_image_count == infos.capabilities.max_image_count {
                infos.capabilities.min_image_count
            } else {
                infos.capabilities.min_image_count + 1
            };

        // SPECIFY : sharing mode
        let image_sharing_mode = if infos.graphics_idx != infos.present_idx {
            SharingMode::CONCURRENT
        } else {
            SharingMode::EXCLUSIVE
        };
        let queue_family_indices = [infos.graphics_idx, infos.present_idx];

        // SPECIFY : behaviour, images
        let create_info = SwapchainCreateInfoKHR::default()
            // Behaviour
            .surface(*surface)
            .present_mode(infos.present_mode)
            .min_image_count(min_image_count)
            // Image : format, extent and usage (= how they will be updated)
            .image_format(infos.surface_format.format)
            .image_color_space(infos.surface_format.color_space)
            .image_extent(infos.capabilities.current_extent)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            // Sharing mode
            .queue_family_indices(&queue_family_indices)
            .image_sharing_mode(image_sharing_mode)
            // Others
            .clipped(true)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .pre_transform(infos.capabilities.current_transform)
            .image_array_layers(1)
            .old_swapchain(SwapchainKHR::null());

        // CREATE : swapchain and images
        let swapchain = unsafe { device.swapchain_khr().create_swapchain(&create_info, None) }
            .expect("Failed to create swapchain.");
        let images = unsafe { device.swapchain_khr().get_swapchain_images(swapchain) }
            .expect("Failed to extract images.");
        Swapchain { swapchain, images }
    }
}
