use std::ops::Deref;

use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Format,
    ImageLayout, PipelineBindPoint, RenderPass, RenderPassCreateInfo, SampleCountFlags,
    SubpassDescription,
};

use super::RendererDevice;

pub struct RendererRenderPass {
    render_pass: RenderPass,
}

// Deref to ash::vk::RenderPass
impl Deref for RendererRenderPass {
    type Target = RenderPass;
    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl RendererRenderPass {
    pub fn new(device: &RendererDevice, format: &Format) -> RendererRenderPass {
        // SPECIFY : out only attachment
        let color_attachment = AttachmentDescription::default()
            .format(*format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        // CREATE : subpass
        let attachment_ref = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments_refs = [attachment_ref];
        let subpass = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments_refs);

        let subpasses = [subpass];
        let color_attachments = [color_attachment];
        let create_info = RenderPassCreateInfo::default()
            .attachments(&color_attachments)
            .subpasses(&subpasses);

        let render_pass = unsafe { device.create_render_pass(&create_info, None) }
            .expect("Failed to create render pass.");

        RendererRenderPass { render_pass }
    }
}
