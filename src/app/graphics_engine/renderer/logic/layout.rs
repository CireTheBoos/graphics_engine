use ash::vk::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
    PipelineLayout, PipelineLayoutCreateInfo, ShaderStageFlags,
};

use crate::app::graphics_engine::Device;

pub struct Layout {
    pub pipeline: PipelineLayout,
    pub mvp: DescriptorSetLayout,
}

impl Layout {
    pub fn new(device: &Device) -> Layout {
        // Sets
        let mvp_layout = create_mvp_layout(device);
        let set_layouts = [mvp_layout];

        // Creation
        let create_info = PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&create_info, None)
                .expect("Failed to create pipeline layout.")
        };
        Layout {
            pipeline: pipeline_layout,
            mvp: mvp_layout,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device.destroy_descriptor_set_layout(self.mvp, None);
            device.destroy_pipeline_layout(self.pipeline, None);
        }
    }
}

fn create_mvp_layout(device: &Device) -> DescriptorSetLayout {
    // Bindings
    let binding = DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_count(1)
        .descriptor_type(DescriptorType::UNIFORM_BUFFER)
        .stage_flags(ShaderStageFlags::VERTEX);
    let bindings = [binding];

    // Creation
    let create_info = DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    unsafe {
        device
            .create_descriptor_set_layout(&create_info, None)
            .expect("Failed to create descriptor set")
    }
}
