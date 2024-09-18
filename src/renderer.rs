mod device;
mod pipeline;
mod swapchain;

use crate::instance::Instance;
use ash::vk::{ImageView, Queue, SurfaceKHR};
pub use device::RendererDevice;
use pipeline::RendererPipeline;
use swapchain::RendererSwapchain;

use std::ptr::NonNull;

// Render images on screen from model data to a given surface
pub struct Renderer {
    instance: NonNull<Instance>,
    surface: SurfaceKHR,
    device: RendererDevice,
    swapchain: RendererSwapchain,
    image_views: Vec<ImageView>,
    graphics_queue: Queue,
    present_queue: Queue,
    pipeline: RendererPipeline,
}

// Destroy views, swapchain, surface (order matters)
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device
                .destroy_pipeline_layout(self.pipeline.layout, None);
            self.device
                .swapchain_khr()
                .destroy_swapchain(*self.swapchain, None);
            self.instance
                .as_ref()
                .surface_khr()
                .destroy_surface(self.surface, None);
        }
    }
}

impl Renderer {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> Renderer {
        // Create device and queues
        let (device, infos) = RendererDevice::new(instance, &surface);
        let graphics_queue = unsafe { device.get_device_queue(device.graphics_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(device.present_idx, 0) };

        // Create swapchain
        let swapchain = RendererSwapchain::new(&device, &surface, infos);

        // Create views
        let image_views = swapchain.get_image_views(&device);

        // Create render pass

        // Create pipeline
        let pipeline = RendererPipeline::new(&device, &swapchain.extent);

        Renderer {
            instance: NonNull::from(instance),
            surface,
            device,
            swapchain,
            image_views,
            graphics_queue,
            present_queue,
            pipeline,
        }
    }
}
