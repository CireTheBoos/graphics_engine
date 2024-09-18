use std::ptr::NonNull;

use ash::vk::{
    ColorComponentFlags, CullModeFlags, Extent2D, FrontFace, Offset2D,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
    PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
    PipelineVertexInputStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, SampleCountFlags,
    ShaderStageFlags, Viewport,
};

use super::{device::RendererDevice, shaders::ShaderManager};

pub struct RendererPipeline {
    device: NonNull<RendererDevice>,
    pub layout: PipelineLayout,
}

impl RendererPipeline {
    pub fn new(device: &RendererDevice, extent: &Extent2D) -> RendererPipeline {
        // compiling shaders
        let shader_manager = ShaderManager::new(device);
        let vertex = shader_manager.vertex();
        let fragment = shader_manager.fragment();

        // SPECIFY : programmable stages
        let vertex_stage_info = PipelineShaderStageCreateInfo::default()
            .module(vertex)
            .stage(ShaderStageFlags::VERTEX)
            .name(c"main");

        let fragment_stage_info = PipelineShaderStageCreateInfo::default()
            .module(fragment)
            .stage(ShaderStageFlags::FRAGMENT)
            .name(c"main");

        let shader_stages = [vertex_stage_info, fragment_stage_info];

        // SPECIFY : fixed funtions stages
        let vertex_input = PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        let input_assembly = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = Viewport::default()
            .x(0.)
            .y(0.)
            .height(extent.height as f32)
            .width(extent.width as f32)
            .min_depth(0.)
            .max_depth(1.);

        let scissor = Rect2D::default()
            .offset(Offset2D::default().x(0).y(0))
            .extent(*extent);

        let rasterizer = PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1);

        // or pass null ptr ?
        let depth_stencil = PipelineDepthStencilStateCreateInfo::default();

        let color_blend_attachment = PipelineColorBlendAttachmentState::default()
            .color_write_mask(ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blend_state = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&[color_blend_attachment]);

        // CREATE : pipeline layout
        let create_info = PipelineLayoutCreateInfo::default();
        let layout = unsafe { device.create_pipeline_layout(&create_info, None) }
            .expect("Failed to create pipeline layout.");

        // CREATE : render pass

        // Cleanup and return
        unsafe { device.destroy_shader_module(vertex, None) };
        unsafe { device.destroy_shader_module(fragment, None) };
        RendererPipeline {
            device: NonNull::from(device),
            layout,
        }
    }
}
