use ash::vk::{
    ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Framebuffer,
    Offset2D, PipelineBindPoint, Rect2D, RenderPassBeginInfo, SubpassContents,
};

use super::{
    pipeline::{RendererPipeline, RendererRenderPass},
    Device,
};

pub struct CommandManager {
    graphics_reuse: CommandPool,
}

impl CommandManager {
    pub fn new(device: &Device) -> CommandManager {
        CommandManager {
            graphics_reuse: create_graphics_reuse_pool(device),
        }
    }

    pub fn graphics_reuse_new_cmdbuf(&self, device: &Device) -> CommandBuffer {
        let allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(self.graphics_reuse)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        unsafe { device.allocate_command_buffers(&allocate_info) }
            .expect("Failed to allocate command buffer.")[0]
        }

    pub fn destroy(&mut self, device: &Device) {
        unsafe { device.destroy_command_pool(self.graphics_reuse, None) };
    }

    // record frame specific cmds on execute_pipeline
    pub fn record_frame(
        &self,
        device: &Device,
        execute_pipeline: &CommandBuffer,
        render_pass: &RendererRenderPass,
        framebuffer: &Framebuffer,
        pipeline: &RendererPipeline,
    ) {
        // BEGIN RECORDING : execute_pipeline
        let begin_info = CommandBufferBeginInfo::default();
        unsafe { device.begin_command_buffer(*execute_pipeline, &begin_info) }
            .expect("Failed to start recording command buffer.");

        // begin render pass
        let mut clear_color = ClearValue::default();
        clear_color.color.float32 = [0., 0., 0., 1.];
        let clear_values = [clear_color];
        let render_pass_begin = RenderPassBeginInfo::default()
            .render_pass(**render_pass)
            .framebuffer(*framebuffer)
            .render_area(
                Rect2D::default()
                    .offset(Offset2D::default())
                    .extent(device.infos.capabilities.current_extent),
            )
            .clear_values(&clear_values);
        unsafe {
            device.cmd_begin_render_pass(
                *execute_pipeline,
                &render_pass_begin,
                SubpassContents::INLINE,
            )
        };

        // bind pipeline
        unsafe {
            device.cmd_bind_pipeline(*execute_pipeline, PipelineBindPoint::GRAPHICS, **pipeline)
        };

        // draw
        unsafe { device.cmd_draw(*execute_pipeline, 3, 1, 0, 0) };

        // end render pass
        unsafe { device.cmd_end_render_pass(*execute_pipeline) };

        // END RECORDING : execute_pipeline
        unsafe { device.end_command_buffer(*execute_pipeline) }
            .expect("Failed to record command buffer.");
    }

}

fn create_graphics_reuse_pool(device: &Device) -> CommandPool {
    let create_info = CommandPoolCreateInfo::default()
        .queue_family_index(device.infos.graphics_idx)
        .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    unsafe { device.create_command_pool(&create_info, None) }
        .expect("Failed to create command pool.")
}