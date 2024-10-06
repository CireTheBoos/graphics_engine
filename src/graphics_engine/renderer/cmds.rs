use ash::vk::{
    BufferCopy, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Offset2D,
    PipelineBindPoint, Rect2D, RenderPassBeginInfo, SubpassContents,
};

use crate::{
    graphics_engine::allocator::Buffer,
    graphics_engine::{Device, FLIGHTS},
    model::{Vertex, MAX_VERTICES},
};

use super::Renderer;

// Pools
pub fn create_graphics_pool(device: &Device) -> CommandPool {
    create_pool(
        device,
        device.infos.graphics_idx,
        CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
    )
}

pub fn create_transfer_pool(device: &Device) -> CommandPool {
    create_pool(
        device,
        device.infos.transfer_idx,
        CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
    )
}

fn create_pool(device: &Device, queue_family: u32, flags: CommandPoolCreateFlags) -> CommandPool {
    let create_info = CommandPoolCreateInfo::default()
        .queue_family_index(queue_family)
        .flags(flags);
    unsafe { device.create_command_pool(&create_info, None) }
        .expect("Failed to create command pool.")
}

// Commands
pub fn allocate_draws(pool: CommandPool, device: &Device) -> Vec<CommandBuffer> {
    let allocate_info = CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(FLIGHTS as u32);
    unsafe { device.allocate_command_buffers(&allocate_info) }
        .expect("Failed to allocate command buffer.")
}

pub fn allocate_record_transfer(
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
    unsafe { device.cmd_copy_buffer(transfer_vertices, **src_buffer, **dst_buffer, &regions) };

    // END
    unsafe { device.end_command_buffer(transfer_vertices) }
        .expect("Failed to record upload_vertices.");

    transfer_vertices
}

impl Renderer {
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
}
