mod mvp;
mod pools;

use ash::vk::{DescriptorPool, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout};
pub use mvp::allocate_configure_mvp_set;
pub use pools::create_uniform_buffer_pool;

use crate::app::graphics_engine::Device;

fn allocate_descriptor_sets(
    device: &Device,
    descriptor_pool: &DescriptorPool,
    set_layouts: &[DescriptorSetLayout],
) -> Vec<DescriptorSet> {
    let allocate_info = DescriptorSetAllocateInfo::default()
        .descriptor_pool(*descriptor_pool)
        .set_layouts(&set_layouts);

    unsafe {
        device
            .allocate_descriptor_sets(&allocate_info)
            .expect("Failed to allocate descriptor set")
    }
}
