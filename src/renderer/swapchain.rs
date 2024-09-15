use std::ops::{Deref, DerefMut};

use ash::{
    khr::swapchain::Device as SwapChainDevice,
    vk::{
        ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Extent2D, Format, Image,
        ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo,
        ImageViewType, SharingMode, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
    Device as AshDevice,
};

use super::PhysicalDeviceInfos;

pub struct RendererSwapchain {
    swapchain: SwapchainKHR,
    pub format: Format,
    pub extent: Extent2D,
    images: Vec<Image>,
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

        let images =
            unsafe { device.get_swapchain_images(swapchain) }.expect("Failed to extract images.");

        RendererSwapchain {
            swapchain,
            format: infos.format.format,
            extent: infos.extent,
            images,
        }
    }

    pub fn get_image_views(&self, device: &AshDevice) -> Vec<ImageView> {
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
                    .format(self.format)
                    .components(components)
                    .subresource_range(subresource_range);

                unsafe { device.create_image_view(&create_info, None) }
                    .expect("Failed to create image view.")
            })
            .collect()
    }
}
