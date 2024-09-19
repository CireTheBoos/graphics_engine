mod device;
mod pipeline;
mod swapchain;

use crate::instance::Instance;
use ash::vk::{Extent2D, Framebuffer, FramebufferCreateInfo, ImageView, Queue, SurfaceKHR};
pub use device::RendererDevice;
use pipeline::{RendererPipeline, RendererRenderPass};
use swapchain::RendererSwapchain;

// Given a surface :
// - Computes imgs from input data (adapted to surface)
// - Presents them continuously on surface
pub struct Renderer {
    surface: SurfaceKHR,
    device: RendererDevice,
    swapchain: RendererSwapchain,
    image_views: Vec<ImageView>,
    graphics_queue: Queue,
    present_queue: Queue,
    pipeline: RendererPipeline,
    frame_buffers: Vec<Framebuffer>,
}

impl Renderer {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> Renderer {
        // Create device and queues
        let device = RendererDevice::new(instance, &surface);
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(device.infos.present_idx, 0) };

        // Create swapchain
        let swapchain = RendererSwapchain::new(&device, &surface);

        // Create views
        let image_views = swapchain.get_image_views(&device);

        // Create pipeline
        let pipeline = RendererPipeline::new(&device, &swapchain);

        // Create frame buffers
        let frame_buffers = create_frame_buffers(
            &image_views,
            &pipeline.render_pass,
            &swapchain.extent,
            &device,
        );

        Renderer {
            surface,
            device,
            swapchain,
            image_views,
            graphics_queue,
            present_queue,
            pipeline,
            frame_buffers,
        }
    }

    // Destroy views, swapchain, surface (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            for framebuffer in &self.frame_buffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.pipeline.destroy(&self.device);
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device
                .swapchain_khr()
                .destroy_swapchain(*self.swapchain, None);
            instance.surface_khr().destroy_surface(self.surface, None);
            self.device.destroy();
        }
    }
}

fn create_frame_buffers(
    image_views: &Vec<ImageView>,
    render_pass: &RendererRenderPass,
    extent: &Extent2D,
    device: &RendererDevice,
) -> Vec<Framebuffer> {
    image_views
        .iter()
        .map(|view| {
            let attachments = [*view];

            let create_info = FramebufferCreateInfo::default()
                .render_pass(render_pass.render_pass)
                .layers(1)
                .height(extent.height)
                .width(extent.width)
                .attachments(&attachments);

            unsafe { device.create_framebuffer(&create_info, None) }
                .expect("Failed to create framebuffer.")
        })
        .collect()
}
