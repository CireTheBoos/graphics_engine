mod pipeline;
mod render_pass;
mod shaders;

use ash::vk::{
    ComponentMapping, Extent2D, Fence, Framebuffer, FramebufferCreateInfo, Image, ImageAspectFlags,
    ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType, PipelineStageFlags,
    Queue, Semaphore, SubmitInfo,
};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;

use crate::graphics_engine::{Commander, Dealer, Device, Presenter, Syncer};

pub struct Renderer {
    transfer_queue: Queue,
    graphics_queue: Queue,
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: Vec<Framebuffer>,
}

impl Renderer {
    pub fn new(device: &Device, presenter: &Presenter) -> Renderer {
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let transfer_queue = unsafe { device.get_device_queue(device.infos.transfer_idx, 0) };
        let image_views = create_image_views(&device, presenter.swapchain_images());
        let render_pass = RenderPass::new(&device);
        let pipeline = Pipeline::new(&device, &render_pass);
        let frame_buffers = create_frame_buffers(
            &image_views,
            &render_pass,
            &device.infos.capabilities.current_extent,
            &device,
        );
        Renderer {
            graphics_queue,
            transfer_queue,
            image_views,
            render_pass,
            pipeline,
            frame_buffers,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            for framebuffer in &self.frame_buffers {
                device.destroy_framebuffer(*framebuffer, None);
            }
            device.destroy_render_pass(*self.render_pass, None);
            self.pipeline.destroy(device);
            for image_view in &self.image_views {
                device.destroy_image_view(*image_view, None);
            }
        }
    }

    pub fn transfer_vertices(
        &self,
        device: &Device,
        commander: &Commander,
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [commander.transfer_vertices];
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe { device.queue_submit(self.transfer_queue, &[submit_info], signal_fence) }
            .expect("Failed to submit upload_vertices cmd buf.");
    }

    pub fn draw(
        &self,
        device: &Device,
        commander: &Commander,
        syncer: &Syncer,
        wait_semaphores: &[Semaphore],
        wait_dst_stage_mask: &[PipelineStageFlags],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [commander.draws[syncer.current_flight().idx]];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers);
        unsafe { device.queue_submit(self.graphics_queue, &[submit_info], signal_fence) }
            .expect("Failed to submit draw cmd buf.");
    }

    pub fn record_draw(
        &self,
        device: &Device,
        syncer: &Syncer,
        dealer: &Dealer,
        commander: &Commander,
    ) {
        commander.record_draw(
            device,
            syncer.current_flight().idx,
            &self.frame_buffers[syncer.current_flight().idx],
            &self.render_pass,
            &self.pipeline,
            &dealer.vertex_buffer,
        );
    }
}

fn create_image_views(device: &Device, images: &Vec<Image>) -> Vec<ImageView> {
    images
        .iter()
        .map(|image| {
            // identity
            let components = ComponentMapping::default();

            let subresource_range = ImageSubresourceRange::default()
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let create_info = ImageViewCreateInfo::default()
                // view restrictions
                .image(*image)
                .view_type(ImageViewType::TYPE_2D)
                .subresource_range(subresource_range)
                // data interpretation
                .format(device.infos.surface_format.format)
                .components(components);

            unsafe { device.create_image_view(&create_info, None) }
                .expect("Failed to create image view.")
        })
        .collect()
}

fn create_frame_buffers(
    image_views: &Vec<ImageView>,
    render_pass: &RenderPass,
    extent: &Extent2D,
    device: &Device,
) -> Vec<Framebuffer> {
    image_views
        .iter()
        .map(|view| {
            let attachments = [*view];

            let create_info = FramebufferCreateInfo::default()
                .render_pass(render_pass.render_pass)
                .layers(1)
                .height(extent.height)
                .width(extent.width)
                .attachments(&attachments);

            unsafe { device.create_framebuffer(&create_info, None) }
                .expect("Failed to create framebuffer.")
        })
        .collect()
}
