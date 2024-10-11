use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel,
    CommandPool, Framebuffer, IndexType, Offset2D, PipelineBindPoint, Rect2D, RenderPassBeginInfo,
    SubpassContents,
};

use crate::graphics_engine::{Device, Renderer};

pub fn allocate_draw(device: &Device, pool: CommandPool) -> CommandBuffer {
    let allocate_info = CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .expect("Failed to allocate command buffer.")[0]
    }
}

impl Renderer {
    pub fn record_draw(&self, device: &Device, swapchain_image_idx: usize) {
        unsafe {
            let framebuffer: &Framebuffer = &self.framebuffers[swapchain_image_idx];

            // Begin
            let begin_info = CommandBufferBeginInfo::default();
            device
                .begin_command_buffer(self.draw, &begin_info)
                .expect("Failed to start recording command buffer.");

            // Begin render pass
            let render_pass_begin = RenderPassBeginInfo::default()
                .render_pass(*self.render_pass)
                .framebuffer(*framebuffer)
                .render_area(
                    Rect2D::default()
                        .offset(Offset2D::default())
                        .extent(device.infos.capabilities.current_extent),
                )
                .clear_values(&self.render_pass.clear_values);
            device.cmd_begin_render_pass(self.draw, &render_pass_begin, SubpassContents::INLINE);

            // Bind : pipeline
            device.cmd_bind_pipeline(self.draw, PipelineBindPoint::GRAPHICS, *self.pipeline);

            // Bind : vertices
            let buffers = [*self.vertices];
            let offsets = [0];
            device.cmd_bind_vertex_buffers(self.draw, 0, &buffers, &offsets);

            // Bind : indices
            device.cmd_bind_index_buffer(self.draw, *self.indices, 0, IndexType::UINT32);

            // Draw
            device.cmd_draw_indexed(self.draw, 6, 1, 0, 0, 0);

            // End render pass
            device.cmd_end_render_pass(self.draw);

            // End
            device
                .end_command_buffer(self.draw)
                .expect("Failed to record command buffer.");
        }
    }
}
