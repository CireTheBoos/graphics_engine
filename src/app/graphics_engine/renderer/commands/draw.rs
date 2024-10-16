use ash::vk::{
    ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, Framebuffer, IndexType, PipelineBindPoint, Rect2D,
    RenderPassBeginInfo, SubpassContents,
};

use crate::app::graphics_engine::{mesher::MAX_INDICES, Device, Renderer};

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
            // Begin
            let begin_info = CommandBufferBeginInfo::default();
            device
                .begin_command_buffer(self.draw, &begin_info)
                .expect("Failed to start recording command buffer.");

            // Begin render pass
            self.cmd_begin_render_pass(device, swapchain_image_idx);

            // Bind : pipeline
            device.cmd_bind_pipeline(self.draw, PipelineBindPoint::GRAPHICS, *self.pipeline);

            // Bind : vertices
            let buffers = [*self.vertices];
            let offsets = [0];
            device.cmd_bind_vertex_buffers(self.draw, 0, &buffers, &offsets);

            // Bind : indices
            device.cmd_bind_index_buffer(self.draw, *self.indices, 0, IndexType::UINT32);

            // Bind : MVP
            let sets = [self.mvp_set];
            device.cmd_bind_descriptor_sets(
                self.draw,
                PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &sets,
                &[],
            );

            // Draw
            device.cmd_draw_indexed(self.draw, MAX_INDICES as u32, 1, 0, 0, 0);

            // End render pass
            device.cmd_end_render_pass(self.draw);

            // End
            device
                .end_command_buffer(self.draw)
                .expect("Failed to record command buffer.");
        }
    }

    fn cmd_begin_render_pass(&self, device: &Device, swapchain_image_idx: usize) {
        // Params
        let framebuffer: &Framebuffer = &self.framebuffers[swapchain_image_idx];
        let render_area = Rect2D::default().extent(device.infos.capabilities.current_extent);
        let clear_values = clear_values();

        // Cmd
        let render_pass_begin = RenderPassBeginInfo::default()
            .render_pass(*self.render_pass)
            .framebuffer(*framebuffer)
            .render_area(render_area)
            .clear_values(&clear_values);
        unsafe {
            device.cmd_begin_render_pass(self.draw, &render_pass_begin, SubpassContents::INLINE)
        };
    }
}

// Clears to black
fn clear_values() -> Vec<ClearValue> {
    let mut clear_color = ClearValue::default();
    clear_color.color.float32 = [0., 0., 0., 1.];
    vec![clear_color]
}
