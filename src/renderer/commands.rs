use ash::vk::{
    ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Framebuffer,
    Offset2D, PipelineBindPoint, Rect2D, RenderPassBeginInfo, SubpassContents,
};

use super::{
    pipeline::{RendererPipeline, RendererRenderPass},
    RendererDevice,
};

pub struct RendererCommands {
    command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
}

impl RendererCommands {
    pub fn new(device: &RendererDevice) -> RendererCommands {
        let command_pool = create_command_pool(device);
        let command_buffer = create_command_buffer(device, &command_pool);
        RendererCommands {
            command_pool,
            command_buffer,
        }
    }

    pub fn destroy(&mut self, device: &RendererDevice) {
        unsafe { device.destroy_command_pool(self.command_pool, None) };
    }

    pub fn record_command_buffer(
        &mut self,
        device: &RendererDevice,
        render_pass: &RendererRenderPass,
        framebuffer: &Framebuffer,
        pipeline: &RendererPipeline,
    ) {
        // begin recording
        let begin_info = CommandBufferBeginInfo::default();
        unsafe { device.begin_command_buffer(self.command_buffer, &begin_info) }
            .expect("Failed to start recording command buffer.");

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

        // begin renderpass
        unsafe {
            device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin,
                SubpassContents::INLINE,
            )
        };

        unsafe {
            device.cmd_bind_pipeline(self.command_buffer, PipelineBindPoint::GRAPHICS, **pipeline)
        };

        unsafe { device.cmd_draw(self.command_buffer, 3, 1, 0, 0) };

        unsafe { device.cmd_end_render_pass(self.command_buffer) };

        unsafe { device.end_command_buffer(self.command_buffer) }
            .expect("Failed to record command buffer.");
    }
}

fn create_command_pool(device: &RendererDevice) -> CommandPool {
    let create_info = CommandPoolCreateInfo::default()
        .queue_family_index(device.infos.graphics_idx)
        .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    unsafe { device.create_command_pool(&create_info, None) }
        .expect("Failed to create command pool.")
}

fn create_command_buffer(device: &RendererDevice, command_pool: &CommandPool) -> CommandBuffer {
    let allocate_info = CommandBufferAllocateInfo::default()
        .command_pool(*command_pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    unsafe { device.allocate_command_buffers(&allocate_info) }
        .expect("Failed to allocate command buffer.")[0]
}
