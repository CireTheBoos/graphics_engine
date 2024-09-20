mod commands;
mod device;
mod pipeline;
mod swapchain;

use crate::instance::Instance;
use ash::vk::{Extent2D, Framebuffer, FramebufferCreateInfo, ImageView, Queue, SurfaceKHR};
use commands::RendererCommands;
pub use device::RendererDevice;
use pipeline::{RendererPipeline, RendererRenderPass};
use swapchain::RendererSwapchain;

// Given a surface :
// - Computes imgs from input data (adapted to the surface)
// - Presents them continuously on the surface
pub struct Renderer {
    surface: SurfaceKHR,
    device: RendererDevice,
    graphics_queue: Queue,
    present_queue: Queue,
    // presentation
    swapchain: RendererSwapchain,
    // computation
    image_views: Vec<ImageView>,
    render_pass: RendererRenderPass,
    pipeline: RendererPipeline,
    frame_buffers: Vec<Framebuffer>,
    // commands
    commands: RendererCommands,
}

impl Renderer {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> Renderer {
        // Create device and queues
        let device = RendererDevice::new(instance, &surface);
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(device.infos.present_idx, 0) };

        // PRESENTATION : Create swapchain
        let swapchain = RendererSwapchain::new(&device, &surface);

        // COMPUTATION : Create pipeline, image views
        let image_views = swapchain.get_image_views(&device);
        let render_pass = RendererRenderPass::new(&device);
        let pipeline = RendererPipeline::new(&device, &render_pass);
        let frame_buffers = create_frame_buffers(
            &image_views,
            &render_pass,
            &device.infos.capabilities.current_extent,
            &device,
        );

        // COMMAND :
        let mut commands = RendererCommands::new(&device);

        // in draw ?
        let idx = 0;
        commands.record_command_buffer(&device, &render_pass, &frame_buffers[idx], &pipeline);

        Renderer {
            surface,
            device,
            graphics_queue,
            present_queue,
            swapchain,
            image_views,
            render_pass,
            pipeline,
            frame_buffers,
            commands,
        }
    }

    // Destroy views, swapchain, surface (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            self.commands.destroy(&self.device);
            for framebuffer in &self.frame_buffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.destroy_render_pass(*self.render_pass, None);
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
