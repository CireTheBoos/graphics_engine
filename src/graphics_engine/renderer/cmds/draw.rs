use ash::vk::{
    ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, Framebuffer, Offset2D, PipelineBindPoint, Rect2D,
    RenderPassBeginInfo, SubpassContents,
};

use crate::graphics_engine::{Device, Renderer};

pub fn allocate_draw(device: &Device, pool: CommandPool) -> CommandBuffer {
    let allocate_info = CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    unsafe { device.allocate_command_buffers(&allocate_info) }
        .expect("Failed to allocate command buffer.")[0]
}

impl Renderer {
    pub fn record_draw(&self, device: &Device, img_idx: usize) {
        let framebuffer: &Framebuffer = &self.framebuffers[img_idx];
        // BEGIN
        let begin_info = CommandBufferBeginInfo::default();
        unsafe { device.begin_command_buffer(self.draw, &begin_info) }
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
            device.cmd_begin_render_pass(self.draw, &render_pass_begin, SubpassContents::INLINE)
        };

        // bind pipeline
        unsafe { device.cmd_bind_pipeline(self.draw, PipelineBindPoint::GRAPHICS, *self.pipeline) };

        // bind vertex buffer
        let vertex_buffers = [*self.vertex_buffer];
        let offsets = [0];
        unsafe { device.cmd_bind_vertex_buffers(self.draw, 0, &vertex_buffers, &offsets) };

        // draw
        unsafe { device.cmd_draw(self.draw, 3, 1, 0, 0) };

        // end render pass
        unsafe { device.cmd_end_render_pass(self.draw) };

        // END
        unsafe { device.end_command_buffer(self.draw) }.expect("Failed to record command buffer.");
    }
}
