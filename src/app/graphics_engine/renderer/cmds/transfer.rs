use ash::vk::{
    Buffer, BufferCopy, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandPool,
};

use crate::app::{
    game::{MAX_INDICES, MAX_VERTICES},
    graphics_engine::{model::Vertex, Device},
};

pub fn allocate_record_transfer(
    device: &Device,
    pool: CommandPool,
    staging_vertices: &Buffer,
    vertices: &Buffer,
    staging_indices: &Buffer,
    indices: &Buffer,
) -> CommandBuffer {
    let transfer = allocate_transfer(device, pool);
    record_transfer(
        device,
        &transfer,
        staging_vertices,
        vertices,
        staging_indices,
        indices,
    );
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
    staging_vertices: &Buffer,
    vertices: &Buffer,
    staging_indices: &Buffer,
    indices: &Buffer,
) {
    // Begin
    let begin_info = CommandBufferBeginInfo::default();
    unsafe {
        device
            .begin_command_buffer(*transfer, &begin_info)
            .expect("Failed to begin transfer.");
    }

    // Copy vertices
    let region = BufferCopy::default() // Offset of 0 for src and dst
        .size(Vertex::size_of() as u64 * MAX_VERTICES);
    let regions = [region];
    unsafe { device.cmd_copy_buffer(*transfer, *staging_vertices, *vertices, &regions) };

    // Copy indices
    let region = BufferCopy::default() // Offset of 0 for src and dst
        .size(size_of::<u32>() as u64 * MAX_INDICES);
    let regions = [region];
    unsafe { device.cmd_copy_buffer(*transfer, *staging_indices, *indices, &regions) };

    // End
    unsafe {
        device
            .end_command_buffer(*transfer)
            .expect("Failed to record transfer.");
    }
}
