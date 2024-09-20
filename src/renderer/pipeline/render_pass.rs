use std::ops::Deref;

use ash::vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    ImageLayout, PipelineBindPoint, PipelineStageFlags, RenderPass, RenderPassCreateInfo,
    SampleCountFlags, SubpassDependency, SubpassDescription, SUBPASS_EXTERNAL,
};

use super::RendererDevice;

pub struct RendererRenderPass {
    pub render_pass: RenderPass,
}

// Deref to ash::vk::RenderPass
impl Deref for RendererRenderPass {
    type Target = RenderPass;
    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl RendererRenderPass {
    pub fn new(device: &RendererDevice) -> RendererRenderPass {
        // SPECIFY : 1 attachment
        let color_attachment = AttachmentDescription::default()
            .format(device.infos.surface_format.format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);
        let color_attachments = [color_attachment];
        let attachment_ref = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachments_refs = [attachment_ref];

        // SPECIFY : subpass dependency
        let dependency = SubpassDependency::default()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::empty())
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE);
        let dependencies = [dependency];

        // CREATE : subpass
        let subpass = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_refs);
        let subpasses = [subpass];

        // CREATE : render pass
        let create_info = RenderPassCreateInfo::default()
            .attachments(&color_attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);
        let render_pass = unsafe { device.create_render_pass(&create_info, None) }
            .expect("Failed to create render pass.");
        RendererRenderPass { render_pass }
    }
}
