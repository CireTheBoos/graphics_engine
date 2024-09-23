mod render_pass;

use std::ops::Deref;

use ash::vk::{
    ColorComponentFlags, CullModeFlags, FrontFace, GraphicsPipelineCreateInfo, Offset2D, Pipeline,
    PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, SampleCountFlags,
    ShaderStageFlags, Viewport,
};
pub use render_pass::RendererRenderPass;

use super::shaders::ShaderManager;
use super::Device;

pub struct RendererPipeline {
    pub layout: PipelineLayout,
    graphics_pipeline: Pipeline,
}

// Deref to ash::vk::Pipeline
impl Deref for RendererPipeline {
    type Target = Pipeline;
    fn deref(&self) -> &Self::Target {
        &self.graphics_pipeline
    }
}

impl RendererPipeline {
    pub fn new(device: &Device, render_pass: &RendererRenderPass) -> RendererPipeline {
        let extent = &device.infos.capabilities.current_extent;

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
        let vertex_input_state = PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        let input_assembly_state = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = Viewport::default()
            .x(0.)
            .y(0.)
            .height(extent.height as f32)
            .width(extent.width as f32)
            .min_depth(0.)
            .max_depth(1.);
        let viewports = [viewport];

        let scissor = Rect2D::default()
            .offset(Offset2D::default().x(0).y(0))
            .extent(*extent);
        let scissors = [scissor];

        let viewport_state = PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1);

        let color_blend_attachment = PipelineColorBlendAttachmentState::default()
            .color_write_mask(ColorComponentFlags::RGBA)
            .blend_enable(false);
        let attachments = [color_blend_attachment];
        let color_blend_state = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&attachments);

        // CREATE : pipeline layout
        let create_info = PipelineLayoutCreateInfo::default();
        let layout = unsafe { device.create_pipeline_layout(&create_info, None) }
            .expect("Failed to create pipeline layout.");

        // CREATE : pipeline

        let pipeline_info = GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(layout)
            .render_pass(render_pass.render_pass)
            .subpass(0);

        let create_info = [pipeline_info];

        let graphics_pipelines =
            unsafe { device.create_graphics_pipelines(PipelineCache::null(), &create_info, None) }
                .expect("Failed to create graphics pipeline.");

        // Cleanup and return
        unsafe { device.destroy_shader_module(vertex, None) };
        unsafe { device.destroy_shader_module(fragment, None) };
        RendererPipeline {
            layout,
            graphics_pipeline: graphics_pipelines[0],
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
        }
    }
}
