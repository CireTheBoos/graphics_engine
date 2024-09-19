use std::ops::Deref;

use ash::{
    vk::{
        ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Extent2D, Format, Image,
        ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo,
        ImageViewType, SharingMode, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
    Device as AshDevice,
};

use super::device::RendererDevice;

pub struct RendererSwapchain {
    swapchain: SwapchainKHR,
    pub format: Format,
    pub extent: Extent2D,
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
        let infos = &device.infos;
        // SPECIFY : minimum image count (+1 above min if possible)
        let min_image_count =
            if infos.capabilities.min_image_count == infos.capabilities.max_image_count {
                infos.capabilities.min_image_count
            } else {
                infos.capabilities.min_image_count + 1
            };

        // SPECIFY : a bunch of things
        let mut create_info = SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .min_image_count(min_image_count)
            .image_format(infos.surface_format.format)
            .image_color_space(infos.surface_format.color_space)
            .image_extent(infos.capabilities.current_extent)
            .image_array_layers(1)
            .present_mode(infos.present_mode)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(infos.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .old_swapchain(SwapchainKHR::null());

        // SPECIFY : queues sharing mode
        let queues = [infos.graphics_idx, infos.present_idx];
        if infos.graphics_idx != infos.present_idx {
            create_info = create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&queues);
        } else {
            create_info = create_info.image_sharing_mode(SharingMode::EXCLUSIVE)
        }

        // CREATE : swapchain
        let swapchain = unsafe { device.swapchain_khr().create_swapchain(&create_info, None) }
            .expect("Failed to create swapchain.");

        // CREATE : images
        let images = unsafe { device.swapchain_khr().get_swapchain_images(swapchain) }
            .expect("Failed to extract images.");

        RendererSwapchain {
            swapchain,
            format: infos.surface_format.format,
            extent: infos.capabilities.current_extent,
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
