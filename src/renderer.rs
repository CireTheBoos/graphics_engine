mod commander;
mod dealer;
mod device;
mod pipeline;
mod render_pass;
mod shaders;
mod swapchain;
mod syncer;

use std::u64;

use crate::instance::Instance;
use ash::vk::{
    Buffer, ComponentMapping, Extent2D, Fence, Framebuffer, FramebufferCreateInfo, Image,
    ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType,
    PipelineStageFlags, PresentInfoKHR, Queue, SubmitInfo, SurfaceKHR,
};
use commander::Commander;
use dealer::{vertices, Dealer};
pub use device::Device;
use pipeline::Pipeline;
use render_pass::RenderPass;
use swapchain::Swapchain;
use syncer::Syncer;
use vk_mem::Allocation;

const FRAMES_IN_FLIGHT: usize = 2;

// Given a surface :
// - Computes imgs from input data (adapted to the surface)
// - Presents them continuously on the surface
pub struct Renderer {
    // Utils
    commander: Commander,
    syncer: Syncer,
    dealer: Dealer,
    // TODO : dealer
    // Presentation
    present_queue: Queue,
    swapchain: Swapchain,
    // Computation
    graphics_queue: Queue,
    vertex_buffer: (Buffer, Allocation),
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: Vec<Framebuffer>,
    // essential objects (drop last bc VMA allocator is freed at drop)
    surface: SurfaceKHR,
    device: Device,
}

impl Renderer {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> Renderer {
        let device = Device::new(instance, &surface);

        // Utils
        let dealer = Dealer::new(instance, &device);
        let commander = Commander::new(&device);
        let syncer = Syncer::new(&device);

        // Presentation
        let swapchain = Swapchain::new(&device, &surface);
        let present_queue = unsafe { device.get_device_queue(device.infos.present_idx, 0) };

        // Computation
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let image_views = create_image_views(&device, &swapchain.images);
        let render_pass = RenderPass::new(&device);
        let pipeline = Pipeline::new(&device, &render_pass);
        let frame_buffers = create_frame_buffers(
            &image_views,
            &render_pass,
            &device.infos.capabilities.current_extent,
            &device,
        );

        let vertices = vertices();
        let mut vertex_buffer = dealer.allocate_vertex_buffer(&device, &vertices);

        let allocation_info = dealer.allocator.get_allocation_info(&vertex_buffer.1);

        let data = unsafe { dealer.allocator.map_memory(&mut vertex_buffer.1) }
            .expect("Failed to map memory.");

        unsafe {
            data.copy_from_nonoverlapping(
                vertices.as_ptr() as *const u8,
                allocation_info.size as usize,
            )
        };

        unsafe { dealer.allocator.unmap_memory(&mut vertex_buffer.1) };

        Renderer {
            surface,
            device,
            commander,
            syncer,
            dealer,
            graphics_queue,
            present_queue,
            vertex_buffer,
            swapchain,
            image_views,
            render_pass,
            pipeline,
            frame_buffers,
        }
    }

    // Destroy views, swapchain, surface (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.dealer
                .allocator
                .destroy_buffer(self.vertex_buffer.0, &mut self.vertex_buffer.1);

            self.syncer.destroy(&self.device);
            self.commander.destroy(&self.device);
            for framebuffer in &self.frame_buffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.destroy_render_pass(*self.render_pass, None);
            self.pipeline.destroy(&self.device);
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device
                .swapchain_khr()
                .destroy_swapchain(*self.swapchain, None);
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn draw_frame(&mut self) {
        // wait for last rendering to finish
        self.syncer
            .wait_in_flight(&self.device, self.syncer.current_frame());

        // Acquiring swapchain img and signal rendering when done
        let (idx, _) = unsafe {
            self.device.swapchain_khr().acquire_next_image(
                *self.swapchain,
                u64::MAX,
                self.syncer.current_frame().img_available,
                Fence::null(),
            )
        }
        .expect("Failed to acquire next swapchain image.");

        // rendering and signal fence and present when done
        self.commander.record_draw(
            &self.device,
            self.syncer.current_frame(),
            &self.frame_buffers[idx as usize],
            &self.render_pass,
            &self.pipeline,
            &self.vertex_buffer.0,
        );
        let wait_semaphores = [self.syncer.current_frame().img_available];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.syncer.current_frame().render_finished];
        let command_buffers = [self.commander.draws[self.syncer.current_frame().idx]];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        unsafe {
            self.device.queue_submit(
                self.graphics_queue,
                &[submit_info],
                self.syncer.current_frame().in_flight,
            )
        }
        .expect("Failed to submit commands.");

        // presentation
        let indices = [idx];
        let swapchains = [*self.swapchain];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);
        unsafe {
            self.device
                .swapchain_khr()
                .queue_present(self.present_queue, &present_info)
        }
        .expect("Failed to present image.");

        self.syncer.step();
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
