use ash::vk::{
    Buffer, BufferCopy, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool,
};

use crate::{
    graphics_engine::Device,
    model::{Vertex, MAX_VERTICES},
};

pub fn allocate_record_transfer(
    device: &Device,
    pool: CommandPool,
    src_buffer: &Buffer,
    dst_buffer: &Buffer,
) -> CommandBuffer {
    let transfer = allocate_transfer(device, pool);
    record_transfer(device, &transfer, src_buffer, dst_buffer);
    transfer
}

fn allocate_transfer(device: &Device, pool: CommandPool) -> CommandBuffer {
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

fn record_transfer(
    device: &Device,
    transfer: &CommandBuffer,
    src_buffer: &Buffer,
    dst_buffer: &Buffer,
) {
    // Begin
    let begin_info = CommandBufferBeginInfo::default();
    unsafe {
        device
            .begin_command_buffer(*transfer, &begin_info)
            .expect("Failed to start recording command buffer.");
    }

    // Copy
    let region = BufferCopy::default() // Offset of 0 for src and dst
        .size(Vertex::size_of() as u64 * MAX_VERTICES);
    let regions = [region];
    unsafe { device.cmd_copy_buffer(*transfer, *src_buffer, *dst_buffer, &regions) };

    // End
    unsafe {
        device
            .end_command_buffer(*transfer)
            .expect("Failed to record upload_vertices.");
    }
}
