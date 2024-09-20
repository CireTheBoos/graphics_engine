mod commands;
mod device;
mod pipeline;
mod swapchain;

use std::u64;

use crate::instance::Instance;
use ash::vk::{
    CommandBufferResetFlags, Extent2D, Fence, FenceCreateFlags, FenceCreateInfo, Framebuffer,
    FramebufferCreateInfo, ImageView, PipelineStageFlags, PresentInfoKHR, Queue, Semaphore,
    SemaphoreCreateInfo, SubmitInfo, SurfaceKHR,
};
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
    img_available_semaphor: Semaphore,
    render_finished_semaphor: Semaphore,
    in_flight_fence: Fence,
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
        let commands = RendererCommands::new(&device);
        let (img_available_semaphor, render_finished_semaphor, in_flight_fence) =
            create_sync_objects(&device);

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
            img_available_semaphor,
            render_finished_semaphor,
            in_flight_fence,
        }
    }

    // Destroy views, swapchain, surface (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device
                .destroy_semaphore(self.img_available_semaphor, None);
            self.device
                .destroy_semaphore(self.render_finished_semaphor, None);
            self.device.destroy_fence(self.in_flight_fence, None);
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

    pub fn draw_frame(&mut self) {
        // wait for previous frame
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight_fence], true, u64::MAX)
                .expect("Failed to wait on previous frame.")
        };
        unsafe { self.device.reset_fences(&[self.in_flight_fence]) }
            .expect("Failed to reset fence.");

        // Acquiring swapchain img
        let (idx, _) = unsafe {
            self.device.swapchain_khr().acquire_next_image(
                *self.swapchain,
                u64::MAX,
                self.img_available_semaphor,
                Fence::null(),
            )
        }
        .expect("Failed to acquire next swapchain image.");

        // Record command buffer
        unsafe {
            self.device.reset_command_buffer(
                self.commands.command_buffer,
                CommandBufferResetFlags::empty(),
            )
        }
        .expect("Failed to reset command buffer.");
        self.commands.record_command_buffer(
            &self.device,
            &self.render_pass,
            &self.frame_buffers[idx as usize],
            &self.pipeline,
        );

        // submit command buffer
        let wait_semaphores = [self.img_available_semaphor];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphor];
        let command_buffers = [self.commands.command_buffer];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        unsafe {
            self.device
                .queue_submit(self.graphics_queue, &[submit_info], self.in_flight_fence)
        }
        .expect("Failed to submit commands.");

        // presentation
        let indices = [idx];
        let swapchains = [*self.swapchain];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);
        unsafe {
            self.device
                .swapchain_khr()
                .queue_present(self.present_queue, &present_info)
        }
        .expect("Failed to present image.");
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

fn create_sync_objects(device: &RendererDevice) -> (Semaphore, Semaphore, Fence) {
    let semaphore_create_info = SemaphoreCreateInfo::default();
    let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);

    let img_available_semaphor = unsafe { device.create_semaphore(&semaphore_create_info, None) }
        .expect("Failed to create semaphore.");
    let render_finished_semaphor = unsafe { device.create_semaphore(&semaphore_create_info, None) }
        .expect("Failed to create semaphore.");
    let in_flight_fence =
        unsafe { device.create_fence(&fence_create_info, None) }.expect("Failed to create fence.");

    (
        img_available_semaphor,
        render_finished_semaphor,
        in_flight_fence,
    )
}
