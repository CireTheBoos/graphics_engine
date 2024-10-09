use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};

use crate::graphics_engine::Device;

fn create_pool(device: &Device, queue_family: u32, flags: CommandPoolCreateFlags) -> CommandPool {
    let create_info = CommandPoolCreateInfo::default()
        .queue_family_index(queue_family)
        .flags(flags);
    unsafe { device.create_command_pool(&create_info, None) }
        .expect("Failed to create command pool.")
}

// graphics pool
pub fn create_graphics_pool(device: &Device) -> CommandPool {
    create_pool(
        device,
        device.infos.graphics_idx,
        CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
    )
}

// transfer pool
pub fn create_transfer_pool(device: &Device) -> CommandPool {
    create_pool(
        device,
        device.infos.transfer_idx,
        CommandPoolCreateFlags::empty(),
    )
}
