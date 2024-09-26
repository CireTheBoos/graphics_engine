use ash::vk::{
    Buffer, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Framebuffer, Offset2D, PipelineBindPoint, Rect2D, RenderPassBeginInfo, SubpassContents
};

use super::{pipeline::Pipeline, render_pass::RenderPass, syncer::Frame, Device};

// Commander translates boilerplate cmd buf code in meaningful fns for renderer. It does NOT submit or sync. :
// - Hold pools
// - Hold multiple-use cmd bufs -> provide fns to rerecord them
// - Allocate temporary cmd bufs
// MULTIPLE-USE CMD LIST :
// - draw : execute the graphics pipeline and render on a given frame buffer
pub struct Commander {
    graphics_pool: CommandPool,
    pub draws: Vec<CommandBuffer>,
}

impl Commander {
    pub fn new(device: &Device) -> Commander {
        let graphics_pool = create_pool(
            device,
            device.infos.graphics_idx,
            CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        );
        let mut draws = Vec::new();
        for _ in 0..super::FRAMES_IN_FLIGHT {
            draws.push(allocate_draw(device, &graphics_pool));
        }
        Commander {
            graphics_pool,
            draws,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe { device.destroy_command_pool(self.graphics_pool, None) };
    }

    pub fn record_draw(
        &self,
        device: &Device,
        frame: &Frame,
        framebuffer: &Framebuffer,
        render_pass: &RenderPass,
        pipeline: &Pipeline,
        vertex_buffer: &Buffer
    ) {
        // BEGIN
        let begin_info = CommandBufferBeginInfo::default();
        unsafe { device.begin_command_buffer(self.draws[frame.idx], &begin_info) }
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
                self.draws[frame.idx],
                &render_pass_begin,
                SubpassContents::INLINE,
            )
        };

        // bind pipeline
        unsafe {
            device.cmd_bind_pipeline(
                self.draws[frame.idx],
                PipelineBindPoint::GRAPHICS,
                **pipeline,
            )
        };

        // bind vertex buffer
        let vertex_buffers = [*vertex_buffer];
        let offsets = [0];
        unsafe { device.cmd_bind_vertex_buffers(self.draws[frame.idx], 0, &vertex_buffers, &offsets) };

        // draw
        unsafe { device.cmd_draw(self.draws[frame.idx], 3, 1, 0, 0) };

        // end render pass
        unsafe { device.cmd_end_render_pass(self.draws[frame.idx]) };

        // END
        unsafe { device.end_command_buffer(self.draws[frame.idx]) }
            .expect("Failed to record command buffer.");
    }
}

fn create_pool(device: &Device, queue_family: u32, flags: CommandPoolCreateFlags) -> CommandPool {
    let create_info = CommandPoolCreateInfo::default()
        .queue_family_index(queue_family)
        .flags(flags);

    unsafe { device.create_command_pool(&create_info, None) }
        .expect("Failed to create command pool.")
}

fn allocate_draw(device: &Device, pool: &CommandPool) -> CommandBuffer {
    let allocate_info = CommandBufferAllocateInfo::default()
        .command_pool(*pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    unsafe { device.allocate_command_buffers(&allocate_info) }
        .expect("Failed to allocate command buffer.")[0]
}
