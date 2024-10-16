use std::ops::Deref;

use crate::app::graphics_engine::{mesher::Vertex, renderer::shaders::Compiler, Device};

use ash::vk::{
    ColorComponentFlags, CullModeFlags, DescriptorSetLayout, FrontFace, GraphicsPipelineCreateInfo,
    Offset2D, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
    PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
    PrimitiveTopology, Rect2D, SampleCountFlags, ShaderStageFlags, Viewport,
};

use super::layout::Layout;

pub struct Pipeline {
    pipeline: ash::vk::Pipeline,
    pub layout: Layout,
}

// Deref to ash::vk::Pipeline
impl Deref for Pipeline {
    type Target = ash::vk::Pipeline;
    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

impl Pipeline {
    pub fn new(device: &Device, render_pass: &ash::vk::RenderPass) -> Pipeline {
        let extent = &device.infos.capabilities.current_extent;

        // compiling shaders
        let shader_compiler = Compiler::new();
        let vertex = shader_compiler.vertex(device);
        let fragment = shader_compiler.fragment(device);

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
        let vertex_binding_descriptions = [Vertex::binding_description()];
        let vertex_attribute_descriptions = Vertex::attribute_description();
        let vertex_input_state = PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);

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

        // Layout
        let layout = Layout::new(device);

        // CREATE : pipeline
        let pipeline_info = GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(layout.pipeline)
            .render_pass(*render_pass)
            .subpass(0);

        let create_info = [pipeline_info];

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(PipelineCache::null(), &create_info, None)
                .expect("Failed to create graphics pipeline.")[0]
        };

        // Cleanup and return
        unsafe { device.destroy_shader_module(vertex, None) };
        unsafe { device.destroy_shader_module(fragment, None) };
        Pipeline { pipeline, layout }
    }

    pub fn mvp_layout(&self) -> &DescriptorSetLayout {
        &self.layout.mvp
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            self.layout.destroy(device);
            device.destroy_pipeline(self.pipeline, None);
        }
    }
}
