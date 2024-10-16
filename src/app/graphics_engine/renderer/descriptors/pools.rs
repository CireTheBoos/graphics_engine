use ash::vk::{
    DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo, DescriptorPoolSize,
    DescriptorType,
};

use crate::app::graphics_engine::Device;

fn create_descriptor_pool(
    device: &Device,
    count: u32,
    ty: DescriptorType,
    flags: DescriptorPoolCreateFlags,
    max_sets: u32,
) -> DescriptorPool {
    let size = DescriptorPoolSize::default().descriptor_count(count).ty(ty);
    let pool_sizes = [size];

    let create_info = DescriptorPoolCreateInfo::default()
        .flags(flags)
        .pool_sizes(&pool_sizes)
        .max_sets(max_sets);

    unsafe {
        device
            .create_descriptor_pool(&create_info, None)
            .expect("Failed to create descriptot pool")
    }
}

pub fn create_uniform_buffer_pool(device: &Device) -> DescriptorPool {
    create_descriptor_pool(
        device,
        1,
        DescriptorType::UNIFORM_BUFFER,
        DescriptorPoolCreateFlags::empty(),
        1,
    )
}
