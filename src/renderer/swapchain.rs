use std::ops::{Deref, DerefMut};

use ash::{
    khr::swapchain::Device as SwapChainDevice,
    vk::{
        CompositeAlphaFlagsKHR, ImageUsageFlags, SharingMode, SurfaceKHR, SwapchainCreateInfoKHR,
        SwapchainKHR,
    },
};

use super::PhysicalDeviceInfos;

pub struct RendererSwapchain {
    swapchain: SwapchainKHR,
}

impl Deref for RendererSwapchain {
    type Target = SwapchainKHR;
    fn deref(&self) -> &Self::Target {
        &self.swapchain
    }
}
impl DerefMut for RendererSwapchain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.swapchain
    }
}

impl RendererSwapchain {
    pub fn new(
        device: &SwapChainDevice,
        surface: &SurfaceKHR,
        infos: PhysicalDeviceInfos,
    ) -> RendererSwapchain {
        // setup the image count (+1 above the min if we can)
        let min_image_count =
            if infos.capabilities.min_image_count == infos.capabilities.max_image_count {
                infos.capabilities.min_image_count
            } else {
                infos.capabilities.min_image_count + 1
            };

        // create swapchain info
        let mut create_info = SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .min_image_count(min_image_count)
            .image_format(infos.format.format)
            .image_color_space(infos.format.color_space)
            .image_extent(infos.extent)
            .image_array_layers(1)
            .present_mode(infos.present_mode)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(infos.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .old_swapchain(SwapchainKHR::null());

        let queues = [infos.graphics_idx, infos.present_idx];
        if infos.graphics_idx != infos.present_idx {
            create_info = create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&queues);
        } else {
            create_info = create_info.image_sharing_mode(SharingMode::EXCLUSIVE)
        }

        let swapchain = unsafe { device.create_swapchain(&create_info, None) }
            .expect("Failed to create swapchain.");

        RendererSwapchain { swapchain }
    }
}
