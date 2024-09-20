use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandPool,
    CommandPoolCreateFlags, CommandPoolCreateInfo,
};

use super::RendererDevice;

pub struct RendererCommands {
    command_pool: CommandPool,
    command_buffer: CommandBuffer,
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
