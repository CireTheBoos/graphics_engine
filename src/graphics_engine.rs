mod commander;
mod dealer;
mod device;
mod pipeline;
mod render_pass;
mod shaders;
mod syncer;
mod presenter;
mod renderer;

use crate::{instance::Instance, model::Vertex};
use ash::vk::{
    ComponentMapping, Extent2D, Fence, Framebuffer, FramebufferCreateInfo, Image, ImageAspectFlags,
    ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType, PipelineStageFlags,
    Queue, Semaphore, SubmitInfo, SurfaceKHR,
};
use commander::Commander;
use dealer::Dealer;
pub use device::Device;
use pipeline::Pipeline;
use presenter::Presenter;
use render_pass::RenderPass;
use syncer::Syncer;

const FLIGHTS: usize = 2;

// Given a surface :
// - Computes imgs from input data (adapted to the surface)
// - Presents them continuously on the surface
pub struct GraphicsEngine {
    // Utils
    commander: Commander,
    syncer: Syncer,
    dealer: Dealer,
    // Presentation
    presenter: Presenter,
    // Computation
    transfer_queue: Queue,
    graphics_queue: Queue,
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: Vec<Framebuffer>,
    // Essentials : Last bc dealer (=> VMA allocator) must drop before
    surface: SurfaceKHR,
    device: Device,
}

impl GraphicsEngine {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> GraphicsEngine {
        // Essentials
        let device = Device::new(instance, &surface);

        // Utils
        let dealer = Dealer::new(instance, &device);
        let commander = Commander::new(&device, &dealer);
        let syncer = Syncer::new(&device);

        // Presentation
        let presenter = Presenter::new(&device, &surface);

        // Computation
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

        GraphicsEngine {
            surface,
            device,
            commander,
            syncer,
            dealer,
            graphics_queue,
            presenter,
            transfer_queue,
            image_views,
            render_pass,
            pipeline,
            frame_buffers,
        }
    }

    // Destroy vulkan objects (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            // Utils
            self.dealer.destroy();
            self.syncer.destroy(&self.device);
            self.commander.destroy(&self.device);

            // Presentation
            self.presenter.destroy(&self.device);

            // Computation
            for framebuffer in &self.frame_buffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.destroy_render_pass(*self.render_pass, None);
            self.pipeline.destroy(&self.device);
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }

            // Essentials
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn render_frame(&mut self, vertices: &Vec<Vertex>) {
        println!("{:#?}",self.syncer);
        //let frame_img_acquired = self.syncer.img_acquired;
        let frame_transfer_done = self.syncer.transfer_done;
        let transfer_done = self.syncer.current_flight().transfer_done;
        let img_available = self.syncer.current_flight().img_available;
        let rendering_done = self.syncer.current_flight().rendering_done;
        let presented = self.syncer.current_flight().presented;

        // WAIT
        let fences = [presented, frame_transfer_done];
        syncer::wait_fences(&self.device, &fences);

        // Update staging vertex buffer
        self.dealer.copy_vertices(vertices);

        // SUBMIT : Transfer
        let signal_semaphores = [transfer_done];
        let signal_fence = frame_transfer_done;
        self.transfer_vertices(&signal_semaphores, signal_fence);

        // Acquire next image
        let signal_semaphore = img_available;
        let signal_fence = Fence::null();
        let image_idx = self.presenter.acquire_next_image(&self.device, signal_semaphore, signal_fence);

        // RECORD : draw
        self.commander.record_draw(
            &self.device,
            self.syncer.current_flight().idx,
            &self.frame_buffers[image_idx as usize],
            &self.render_pass,
            &self.pipeline,
            &self.dealer.vertex_buffer,
        );

        // SUBMIT : draw
        let wait_semaphores = [img_available, transfer_done];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [rendering_done];
        let signal_fence = presented;
        self.draw(
            &wait_semaphores,
            &wait_dst_stage_mask,
            &signal_semaphores,
            signal_fence,
        );

        // PRESENT
        let wait_semaphores = [rendering_done];
        self.presenter.present(&self.device, image_idx, &wait_semaphores);

        self.syncer.step_flight();
    }

    fn transfer_vertices(&self, signal_semaphores: &[Semaphore], signal_fence: Fence) {
        let command_buffers = [self.commander.transfer_vertices];
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe {
            self.device
                .queue_submit(self.transfer_queue, &[submit_info], signal_fence)
        }
        .expect("Failed to submit upload_vertices cmd buf.");
    }

    fn draw(
        &self,
        wait_semaphores: &[Semaphore],
        wait_dst_stage_mask: &[PipelineStageFlags],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [self.commander.draws[self.syncer.current_flight().idx]];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers);
        unsafe {
            self.device
                .queue_submit(self.graphics_queue, &[submit_info], signal_fence)
        }
        .expect("Failed to submit draw cmd buf.");
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
