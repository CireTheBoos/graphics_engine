mod cmds;
mod pipelines;
mod rscs;
mod shaders;

use ash::vk::{
    CommandBuffer, CommandPool, Fence, PipelineStageFlags, Queue, Semaphore, SubmitInfo,
};
use pipelines::{GraphicsFramebuffer, Pipeline, RenderPass};
use vk_mem::Allocator;

use crate::{
    boilerplate::new_semaphore,
    graphics_engine::{Device, Presenter},
    model::Vertex,
};

use super::device::CustomBuffer;

pub struct Renderer {
    // Queues
    transfer_queue: Queue,
    graphics_queue: Queue,
    // Pipeline
    render_pass: RenderPass,
    framebuffers: Vec<GraphicsFramebuffer>,
    pipeline: Pipeline,
    // Rscs
    vertex_buffer: CustomBuffer,
    staging_vertex_buffer: CustomBuffer,
    // Cmds
    graphics_pool: CommandPool,
    transfer_pool: CommandPool,
    draw: CommandBuffer,
    transfer: CommandBuffer,
    // Syncs
    transfer_done: Semaphore,
}

impl Renderer {
    pub fn new(device: &Device, presenter: &Presenter) -> Renderer {
        // Queues
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let transfer_queue = unsafe { device.get_device_queue(device.infos.transfer_idx, 0) };

        // Pipeline
        let render_pass = RenderPass::new(device);
        let framebuffers =
            GraphicsFramebuffer::new(device, &render_pass, presenter.swapchain_images());
        let pipeline = Pipeline::new(device, &render_pass);

        // Rscs
        let vertex_buffer = rscs::allocate_vertex_buffer(device.allocator(), device);
        let staging_vertex_buffer =
            rscs::allocate_staging_vertex_buffer(device.allocator(), device);

        // Cmds
        let graphics_pool = cmds::create_graphics_pool(device);
        let transfer_pool = cmds::create_transfer_pool(device);
        let draw = cmds::allocate_draw(device, graphics_pool);
        let transfer = cmds::allocate_record_transfer(
            device,
            transfer_pool,
            &staging_vertex_buffer,
            &vertex_buffer,
        );

        // Syncs
        let transfer_done = new_semaphore(device);

        Renderer {
            graphics_queue,
            transfer_queue,
            render_pass,
            framebuffers,
            pipeline,
            vertex_buffer,
            staging_vertex_buffer,
            graphics_pool,
            transfer_pool,
            draw,
            transfer,
            transfer_done,
        }
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        unsafe {
            // Cmds
            device.destroy_command_pool(self.graphics_pool, None);
            device.destroy_command_pool(self.transfer_pool, None);

            // Rscs
            self.vertex_buffer.destroy(allocator);
            self.staging_vertex_buffer.destroy(allocator);

            // Pipeline
            for framebuffer in &mut self.framebuffers {
                framebuffer.destroy(device);
            }
            self.pipeline.destroy(device);
            device.destroy_render_pass(*self.render_pass, None);

            // Syncs
            device.destroy_semaphore(self.transfer_done, None);
        }
    }

    pub fn submit_render(
        &mut self,
        device: &Device,
        vertices: &Vec<Vertex>,
        image_idx: u32,
        image_available: Semaphore,
        rendering_done: Semaphore,
        fence_rendering_done: Fence,
    ) {
        // CPU COPY : From CPU to staging vertex buffer
        self.copy_vertices(device, vertices);

        // SUBMIT : transfer
        let signal_semaphores = [self.transfer_done];
        self.submit_transfer(device, &signal_semaphores);

        // RECORD : draw
        self.record_draw(device, image_idx as usize);

        // SUBMIT : draw
        let wait_semaphores = [self.transfer_done, image_available];
        let wait_dst_stage_mask = [
            PipelineStageFlags::VERTEX_INPUT,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ];
        let signal_semaphores = [rendering_done];
        let signal_fence = fence_rendering_done;
        self.submit_draw(
            device,
            &wait_semaphores,
            &wait_dst_stage_mask,
            &signal_semaphores,
            signal_fence,
        );
    }

    fn copy_vertices(&mut self, device: &Device, vertices: &Vec<Vertex>) {
        unsafe {
            device
                .allocator()
                .map_memory(&mut self.staging_vertex_buffer.allocation)
                .expect("Failed to map memory.")
                .copy_from(
                    vertices.as_ptr() as *const u8,
                    Vertex::size_of() * vertices.len(),
                );
            device
                .allocator()
                .unmap_memory(&mut self.staging_vertex_buffer.allocation);
        }
    }

    fn submit_transfer(&self, device: &Device, signal_semaphores: &[Semaphore]) {
        let command_buffers = [self.transfer];
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe { device.queue_submit(self.transfer_queue, &[submit_info], Fence::null()) }
            .expect("Failed to submit transfer.");
    }

    fn submit_draw(
        &self,
        device: &Device,
        wait_semaphores: &[Semaphore],
        wait_dst_stage_mask: &[PipelineStageFlags],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [self.draw];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .signal_semaphores(&signal_semaphores)
            .command_buffers(&command_buffers);
        unsafe { device.queue_submit(self.graphics_queue, &[submit_info], signal_fence) }
            .expect("Failed to submit draw.");
    }
}
