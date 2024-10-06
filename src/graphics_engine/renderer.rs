mod pipeline;
mod render_pass;
mod shaders;

use crate::allocator::Buffer as CustomBuffer;
use ash::vk::{
    Buffer, BufferCopy, BufferCreateInfo, BufferUsageFlags, ClearValue, CommandBuffer,
    CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandPool,
    CommandPoolCreateFlags, CommandPoolCreateInfo, ComponentMapping, Extent2D, Fence, Framebuffer,
    FramebufferCreateInfo, Image, ImageAspectFlags, ImageSubresourceRange, ImageView,
    ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags, Offset2D, PipelineBindPoint,
    PipelineStageFlags, Queue, Rect2D, RenderPassBeginInfo, Semaphore, SharingMode, SubmitInfo,
    SubpassContents,
};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
use vk_mem::{Alloc, AllocationCreateInfo, Allocator};

use crate::{
    graphics_engine::{Device, Presenter},
    model::{Vertex, MAX_VERTICES},
};

use super::FLIGHTS;

pub struct Renderer {
    transfer_queue: Queue,
    graphics_queue: Queue,
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: Vec<Framebuffer>,

    vertex_buffer: CustomBuffer,
    staging_vertex_buffer: CustomBuffer,

    graphics_pool: CommandPool,
    transfer_pool: CommandPool,
    pub draws: Vec<CommandBuffer>,
    pub transfer_vertices: CommandBuffer,
}

impl Renderer {
    pub fn new(device: &Device, presenter: &Presenter, allocator: &Allocator) -> Renderer {
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let transfer_queue = unsafe { device.get_device_queue(device.infos.transfer_idx, 0) };
        let image_views = create_image_views(&device, presenter.swapchain_images());
        let render_pass = RenderPass::new(&device);
        let pipeline = Pipeline::new(&device, &render_pass);
        let frame_buffers = create_frame_buffers(
            &image_views,
            &render_pass,
            &device.infos.capabilities.current_extent,
            &device,
        );

        // Allocate buffers
        let vertex_buffer = allocate_vertex_buffer(allocator, device);
        let staging_vertex_buffer = allocate_staging_vertex_buffer(allocator, device);

        // Pools
        let graphics_pool = create_pool(
            device,
            device.infos.graphics_idx,
            CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        );
        let transfer_pool = create_pool(
            device,
            device.infos.transfer_idx,
            CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        );

        // Command buffers
        let draws = Renderer::draws(graphics_pool, device);
        let transfer_vertices = Renderer::transfer_vertices(
            transfer_pool,
            device,
            &staging_vertex_buffer,
            &vertex_buffer,
        );

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
        }
    }

    // allocate
    fn draws(pool: CommandPool, device: &Device) -> Vec<CommandBuffer> {
        let allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(FLIGHTS as u32);
        unsafe { device.allocate_command_buffers(&allocate_info) }
            .expect("Failed to allocate command buffer.")
    }

    // allocate and record
    fn transfer_vertices(
        pool: CommandPool,
        device: &Device,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
    ) -> CommandBuffer {
        let allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let transfer_vertices = unsafe { device.allocate_command_buffers(&allocate_info) }
            .expect("Failed to allocate command buffer.")[0];

        // BEGIN
        let begin_info = CommandBufferBeginInfo::default();
        unsafe { device.begin_command_buffer(transfer_vertices, &begin_info) }
            .expect("Failed to start recording command buffer.");

        // Copy
        let regions = [BufferCopy::default().size(Vertex::size_of() as u64 * MAX_VERTICES)];
        unsafe { device.cmd_copy_buffer(transfer_vertices, *src_buffer, *dst_buffer, &regions) };

        // END
        unsafe { device.end_command_buffer(transfer_vertices) }
            .expect("Failed to record upload_vertices.");

        transfer_vertices
    }

    pub fn record_draw(&self, device: &Device, flight_idx: usize) {
        let framebuffer = &self.frame_buffers[flight_idx];
        // BEGIN
        let begin_info = CommandBufferBeginInfo::default();
        unsafe { device.begin_command_buffer(self.draws[flight_idx], &begin_info) }
            .expect("Failed to start recording command buffer.");

        // begin render pass
        let mut clear_color = ClearValue::default();
        clear_color.color.float32 = [0., 0., 0., 1.];
        let clear_values = [clear_color];
        let render_pass_begin = RenderPassBeginInfo::default()
            .render_pass(*self.render_pass)
            .framebuffer(*framebuffer)
            .render_area(
                Rect2D::default()
                    .offset(Offset2D::default())
                    .extent(device.infos.capabilities.current_extent),
            )
            .clear_values(&clear_values);
        unsafe {
            device.cmd_begin_render_pass(
                self.draws[flight_idx],
                &render_pass_begin,
                SubpassContents::INLINE,
            )
        };

        // bind pipeline
        unsafe {
            device.cmd_bind_pipeline(
                self.draws[flight_idx],
                PipelineBindPoint::GRAPHICS,
                *self.pipeline,
            )
        };

        // bind vertex buffer
        let vertex_buffers = [*self.vertex_buffer];
        let offsets = [0];
        unsafe {
            device.cmd_bind_vertex_buffers(self.draws[flight_idx], 0, &vertex_buffers, &offsets)
        };

        // draw
        unsafe { device.cmd_draw(self.draws[flight_idx], 3, 1, 0, 0) };

        // end render pass
        unsafe { device.cmd_end_render_pass(self.draws[flight_idx]) };

        // END
        unsafe { device.end_command_buffer(self.draws[flight_idx]) }
            .expect("Failed to record command buffer.");
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
        }
    }

    pub fn transfer(&self, device: &Device, signal_semaphores: &[Semaphore], signal_fence: Fence) {
        let command_buffers = [self.transfer_vertices];
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe { device.queue_submit(self.transfer_queue, &[submit_info], signal_fence) }
            .expect("Failed to submit upload_vertices cmd buf.");
    }

    pub fn draw(
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

    pub fn copy_vertices(&mut self, vertices: &Vec<Vertex>, allocator: &Allocator) {
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

fn create_image_views(device: &Device, images: &Vec<Image>) -> Vec<ImageView> {
    images
        .iter()
        .map(|image| {
            // identity
            let components = ComponentMapping::default();

            let subresource_range = ImageSubresourceRange::default()
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let create_info = ImageViewCreateInfo::default()
                // view restrictions
                .image(*image)
                .view_type(ImageViewType::TYPE_2D)
                .subresource_range(subresource_range)
                // data interpretation
                .format(device.infos.surface_format.format)
                .components(components);

            unsafe { device.create_image_view(&create_info, None) }
                .expect("Failed to create image view.")
        })
        .collect()
}

fn create_frame_buffers(
    image_views: &Vec<ImageView>,
    render_pass: &RenderPass,
    extent: &Extent2D,
    device: &Device,
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

fn allocate_vertex_buffer(allocator: &Allocator, device: &Device) -> CustomBuffer {
    let queue_family_indices = [device.infos.graphics_idx, device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::CONCURRENT)
        .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
        .usage(BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::TRANSFER_DST);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
        ..Default::default()
    };

    let (buffer, allocation) = unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.");

    CustomBuffer { buffer, allocation }
}

fn allocate_staging_vertex_buffer(allocator: &Allocator, device: &Device) -> CustomBuffer {
    let queue_family_indices = [device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
        .usage(BufferUsageFlags::TRANSFER_SRC);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        ..Default::default()
    };

    let (buffer, allocation) = unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.");

    CustomBuffer { buffer, allocation }
}

fn create_pool(device: &Device, queue_family: u32, flags: CommandPoolCreateFlags) -> CommandPool {
    let create_info = CommandPoolCreateInfo::default()
        .queue_family_index(queue_family)
        .flags(flags);
    unsafe { device.create_command_pool(&create_info, None) }
        .expect("Failed to create command pool.")
}
