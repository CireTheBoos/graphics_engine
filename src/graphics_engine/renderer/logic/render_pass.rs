use std::ops::Deref;

use ash::vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    ImageLayout, PipelineBindPoint, PipelineStageFlags, RenderPassCreateInfo, SampleCountFlags,
    SubpassDependency, SubpassDescription, SUBPASS_EXTERNAL,
};

use crate::graphics_engine::Device;

pub struct RenderPass {
    render_pass: ash::vk::RenderPass,
}

// Deref : ash::vk::RenderPass
impl Deref for RenderPass {
    type Target = ash::vk::RenderPass;
    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl RenderPass {
    pub fn new(device: &Device) -> RenderPass {
        // Attachments
        let final_image = AttachmentDescription::default()
            .format(device.infos.surface_format.format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);
        let attachments = [final_image];

        // Subpasses
        let final_image_ref = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachments = [final_image_ref];
        let color_rendering = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments);
        let subpasses = [color_rendering];

        // Dependencies
        let dependency = SubpassDependency::default()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::empty())
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE);
        let dependencies = [dependency];

        // Create Render pass
        let create_info = RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);
        let render_pass = unsafe {
            device
                .create_render_pass(&create_info, None)
                .expect("Failed to create render pass.")
        };

        RenderPass { render_pass }
    }
}
