mod pipeline;
mod render_pass;
mod shaders;

use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, ComponentMapping, Extent2D, Fence, Framebuffer, FramebufferCreateInfo, Image, ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags, PipelineStageFlags, Queue, Semaphore, SharingMode, SubmitInfo
};
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
use vk_mem::{Alloc, Allocation, AllocationCreateInfo, Allocator};

use crate::{graphics_engine::{Commander, Device, Presenter, Syncer}, model::{Vertex, MAX_VERTICES}};

pub struct Renderer {
    transfer_queue: Queue,
    graphics_queue: Queue,
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: Vec<Framebuffer>,

    pub vertex_buffer: Buffer,
    vertex_allocation: Allocation,
    pub staging_vertex_buffer: Buffer,
    staging_vertex_allocation: Allocation,
}

impl Renderer {
    pub fn new(device: &Device, presenter: &Presenter, allocator: &Allocator) -> Renderer {
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

        // Allocate buffers
        let (vertex_buffer, vertex_allocation) = allocate_vertex_buffer(allocator, device);
        let (staging_vertex_buffer, staging_vertex_allocation) =
            allocate_staging_vertex_buffer(allocator, device);

        Renderer {
            graphics_queue,
            transfer_queue,
            image_views,
            render_pass,
            pipeline,
            frame_buffers,

            vertex_buffer,
            vertex_allocation,
            staging_vertex_buffer,
            staging_vertex_allocation,
        }
    }

    pub fn destroy(&mut self, device: &Device, allocator: &Allocator) {
        unsafe {
            allocator
                .destroy_buffer(self.vertex_buffer, &mut self.vertex_allocation);
            allocator.destroy_buffer(
                self.staging_vertex_buffer,
                &mut self.staging_vertex_allocation,
            );
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
        commander: &Commander,
    ) {
        commander.record_draw(
            device,
            syncer.current_flight().idx,
            &self.frame_buffers[syncer.current_flight().idx],
            &self.render_pass,
            &self.pipeline,
            &self.vertex_buffer,
        );
    }

    pub fn copy_vertices(&mut self, vertices: &Vec<Vertex>, allocator: &Allocator) {
        unsafe {
            let staging_vertices = allocator
                .map_memory(&mut self.staging_vertex_allocation)
                .expect("Failed to map memory.");
            staging_vertices.copy_from(
                vertices.as_ptr() as *const u8,
                Vertex::size_of() * vertices.len(),
            );
            allocator
                .unmap_memory(&mut self.staging_vertex_allocation);
        }
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

fn allocate_vertex_buffer(allocator: &Allocator, device: &Device) -> (Buffer, Allocation) {
    let queue_family_indices = [device.infos.graphics_idx, device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::CONCURRENT)
        .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
        .usage(BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::TRANSFER_DST);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
        ..Default::default()
    };

    unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.")
}

fn allocate_staging_vertex_buffer(allocator: &Allocator, device: &Device) -> (Buffer, Allocation) {
    let queue_family_indices = [device.infos.transfer_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .size(size_of::<Vertex>() as u64 * MAX_VERTICES)
        .usage(BufferUsageFlags::TRANSFER_SRC);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        ..Default::default()
    };

    unsafe { allocator.create_buffer(&buffer_info, &create_info) }
        .expect("Failed to create vertex buffer.")
}
