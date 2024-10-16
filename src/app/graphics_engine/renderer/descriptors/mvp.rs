use ash::vk::{
    Buffer, DescriptorBufferInfo, DescriptorPool, DescriptorSet, DescriptorSetLayout,
    DescriptorType, WriteDescriptorSet, WHOLE_SIZE,
};

use crate::app::graphics_engine::Device;

pub fn allocate_configure_mvp_set(
    device: &Device,
    descriptor_pool: &DescriptorPool,
    set_layouts: &[DescriptorSetLayout],
    buffer: &Buffer,
) -> DescriptorSet {
    let set = device.bp_allocate_descriptor_sets(descriptor_pool, set_layouts)[0];
    let buffer_info = DescriptorBufferInfo::default()
        .buffer(*buffer)
        .offset(0)
        .range(WHOLE_SIZE);
    let buffer_infos = [buffer_info];
    let write = WriteDescriptorSet::default()
        .buffer_info(&buffer_infos)
        .dst_set(set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1);
    let descriptor_writes = [write];

    unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    set
}
