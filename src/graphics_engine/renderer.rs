mod cmds;
mod pipeline;
mod render_pass;
mod rscs;
mod shaders;

use super::allocator::Buffer as CustomBuffer;
use ash::vk::{
    CommandBuffer, CommandPool, Fence, Framebuffer, ImageView, PipelineStageFlags, Queue,
    Semaphore, SubmitInfo,
};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
use vk_mem::Allocator;

use crate::{
    boilerplate::new_semaphore,
    graphics_engine::{Device, Presenter},
    model::Vertex,
};

pub struct Renderer {
    // queues
    transfer_queue: Queue,
    graphics_queue: Queue,
    // rscs
    vertex_buffer: CustomBuffer,
    staging_vertex_buffer: CustomBuffer,
    image_views: Vec<ImageView>,
    frame_buffers: Vec<Framebuffer>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    // cmds
    graphics_pool: CommandPool,
    transfer_pool: CommandPool,
    pub draws: Vec<CommandBuffer>,
    pub transfer_vertices: CommandBuffer,
    // sync
    transfer_done: Semaphore,
}

impl Renderer {
    pub fn new(device: &Device, presenter: &Presenter, allocator: &Allocator) -> Renderer {
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let transfer_queue = unsafe { device.get_device_queue(device.infos.transfer_idx, 0) };
        let image_views = rscs::create_image_views(&device, presenter.swapchain_images());
        let render_pass = RenderPass::new(&device);
        let pipeline = Pipeline::new(&device, &render_pass);
        let frame_buffers = rscs::create_frame_buffers(
            &image_views,
            &render_pass,
            &device.infos.capabilities.current_extent,
            &device,
        );

        // Allocate buffers
        let vertex_buffer = rscs::allocate_vertex_buffer(allocator, device);
        let staging_vertex_buffer = rscs::allocate_staging_vertex_buffer(allocator, device);

        // Pools
        let graphics_pool = cmds::create_graphics_pool(device);
        let transfer_pool = cmds::create_transfer_pool(device);

        // Command buffers
        let draws = cmds::allocate_draws(graphics_pool, device);
        let transfer_vertices = cmds::allocate_record_transfer(
            transfer_pool,
            device,
            &staging_vertex_buffer,
            &vertex_buffer,
        );

        let transfer_done = new_semaphore(&device);

        Renderer {
            graphics_queue,
            transfer_queue,
            image_views,
            render_pass,
            pipeline,
            frame_buffers,

            vertex_buffer,
            staging_vertex_buffer,

            graphics_pool,
            transfer_pool,
            draws,
            transfer_vertices,

            transfer_done,
        }
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        unsafe {
            device.destroy_command_pool(self.graphics_pool, None);
            device.destroy_command_pool(self.transfer_pool, None);

            self.vertex_buffer.destroy(allocator);
            self.staging_vertex_buffer.destroy(allocator);

            for framebuffer in &self.frame_buffers {
                device.destroy_framebuffer(*framebuffer, None);
            }
            device.destroy_render_pass(*self.render_pass, None);
            self.pipeline.destroy(device);
            for image_view in &self.image_views {
                device.destroy_image_view(*image_view, None);
            }

            device.destroy_semaphore(self.transfer_done, None);
        }
    }

    pub fn submit_render(
        &mut self,
        vertices: &Vec<Vertex>,
        device: &Device,
        allocator: &Allocator,
        img_available: &[Semaphore],
        render_finished: &[Semaphore],
        presented: Fence,
    ) {
        // Update staging vertex buffer
        self.copy_vertices(vertices, allocator);

        // SUBMIT : Transfer
        let signal_semaphores = [self.transfer_done];
        let signal_fence = Fence::null();
        self.submit_transfer(device, &signal_semaphores, signal_fence);

        // RECORD : draw
        self.record_draw(device, 0);

        // SUBMIT : draw
        let wait_semaphores = [img_available[0], self.transfer_done];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [render_finished[0]];
        let signal_fence = presented;
        self.submit_draw(
            device,
            &wait_semaphores,
            &wait_dst_stage_mask,
            &signal_semaphores,
            signal_fence,
        );
    }

    fn submit_transfer(
        &self,
        device: &Device,
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [self.transfer_vertices];
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe { device.queue_submit(self.transfer_queue, &[submit_info], signal_fence) }
            .expect("Failed to submit upload_vertices cmd buf.");
    }

    fn submit_draw(
        &self,
        device: &Device,
        wait_semaphores: &[Semaphore],
        wait_dst_stage_mask: &[PipelineStageFlags],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [self.draws[0]];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers);
        unsafe { device.queue_submit(self.graphics_queue, &[submit_info], signal_fence) }
            .expect("Failed to submit draw cmd buf.");
    }

    fn copy_vertices(&mut self, vertices: &Vec<Vertex>, allocator: &Allocator) {
        unsafe {
            let staging_vertices = allocator
                .map_memory(&mut self.staging_vertex_buffer.allocation)
                .expect("Failed to map memory.");
            staging_vertices.copy_from(
                vertices.as_ptr() as *const u8,
                Vertex::size_of() * vertices.len(),
            );
            allocator.unmap_memory(&mut self.staging_vertex_buffer.allocation);
        }
    }
}
