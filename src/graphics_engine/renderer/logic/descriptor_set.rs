use ash::vk::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
    ShaderStageFlags,
};

use crate::graphics_engine::Device;

pub fn create_mvp_layout(device: &Device) -> DescriptorSetLayout {
    let binding = DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_count(1)
        .descriptor_type(DescriptorType::UNIFORM_BUFFER)
        .stage_flags(ShaderStageFlags::VERTEX);
    let bindings = [binding];

    let create_info = DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&create_info, None)
            .expect("Failed to create descriptor set")
    }
}
