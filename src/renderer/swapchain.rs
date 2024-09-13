use crate::instance::Instance;
use std::ops::{Deref, DerefMut};

use ash::vk::{Extent2D, Format, PhysicalDevice, PresentModeKHR, SurfaceKHR, SwapchainKHR};

use super::device::RendererDevice;

pub struct RendererSwapchain {
    pub swapchain: SwapchainKHR,
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
        instance: &Instance,
        pĥysical_device: &PhysicalDevice,
        surface: &SurfaceKHR,
    ) -> RendererSwapchain {
        let available_formats = unsafe {
            instance
                .surface_khr()
                .get_physical_device_surface_formats(*physical_device, *surface)
                .unwrap()
        };
    }
}
