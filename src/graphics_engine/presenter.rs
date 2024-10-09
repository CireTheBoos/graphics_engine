mod swapchain;

use crate::graphics_engine::Device;
use ash::vk::{Fence, Image, PresentInfoKHR, Queue, Semaphore, SurfaceKHR};
use swapchain::Swapchain;

// Handles presentation :
// - Hold Swapchain
pub struct Presenter {
    swapchain: Swapchain,
    present_queue: Queue,
}

impl Presenter {
    pub fn new(device: &Device, surface: &SurfaceKHR) -> Presenter {
        let swapchain = Swapchain::new(&device, &surface);
        let present_queue = unsafe { device.get_device_queue(device.infos.present_idx, 0) };
        Presenter {
            swapchain,
            present_queue,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device
                .swapchain_khr()
                .destroy_swapchain(*self.swapchain, None);
        }
    }

    pub fn swapchain_images(&self) -> &Vec<Image> {
        &self.swapchain.images
    }

    pub fn acquire_next_image(&self, device: &Device, image_available: Semaphore) -> (u32, bool) {
        unsafe {
            device
                .swapchain_khr()
                .acquire_next_image(*self.swapchain, u64::MAX, image_available, Fence::null())
                .expect("Failed to acquire next swapchain image.")
        }
    }

    pub fn present(&self, device: &Device, image_idx: u32, rendering_done: Semaphore) {
        let swapchains = [*self.swapchain];
        let indices = [image_idx];
        let wait_semaphores = [rendering_done];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);
        unsafe {
            device
                .swapchain_khr()
                .queue_present(self.present_queue, &present_info)
        }
        .expect("Failed to present image.");
    }
}
