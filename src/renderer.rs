mod device;
mod pipeline;
mod render_pass;
mod swapchain;

use crate::instance::Instance;
use ash::vk::{ImageView, Queue, SurfaceKHR};
pub use device::RendererDevice;
use pipeline::RendererPipeline;
use render_pass::RendererRenderPass;
use swapchain::RendererSwapchain;

// Render images on screen from model data to a given surface
pub struct Renderer {
    surface: SurfaceKHR,
    device: RendererDevice,
    swapchain: RendererSwapchain,
    image_views: Vec<ImageView>,
    graphics_queue: Queue,
    present_queue: Queue,
    pipeline: RendererPipeline,
    render_pass: RendererRenderPass,
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
        let render_pass = RendererRenderPass::new(&device, &swapchain.format);

        // Create pipeline
        let pipeline = RendererPipeline::new(&device, &swapchain.extent);

        Renderer {
            surface,
            device,
            swapchain,
            image_views,
            graphics_queue,
            present_queue,
            pipeline,
            render_pass,
        }
    }

    // Destroy views, swapchain, surface (order matters)
    pub fn destroy(&self, instance: &Instance) {
        unsafe {
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device
                .destroy_pipeline_layout(self.pipeline.layout, None);
            self.device.destroy_render_pass(*self.render_pass, None);
            self.device
                .swapchain_khr()
                .destroy_swapchain(*self.swapchain, None);
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }
}
