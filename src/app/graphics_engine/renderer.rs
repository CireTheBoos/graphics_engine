mod commands;
mod descriptors;
mod logic;
mod resources;
mod shaders;

use ash::vk::{
    CommandBuffer, CommandPool, DescriptorPool, DescriptorSet, Extent2D, Fence, Framebuffer, Image,
    ImageView, PipelineStageFlags, Queue, Semaphore, SubmitInfo,
};
use glam::Mat4;
use logic::{create_framebuffers, Pipeline, RenderPass};
use resources::MVP;

use crate::app::{graphics_engine::Device, model::Camera};

use super::{
    device::{Buffer, MappedBuffer},
    mesher::{Mesh, Vertex},
};

pub struct Renderer {
    // Queues
    transfer_queue: Queue,
    graphics_queue: Queue,
    // Resources
    swapchain_image_views: Vec<ImageView>,
    vertices: Buffer,
    staging_vertices: Buffer,
    indices: Buffer,
    staging_indices: Buffer,
    mvp: MappedBuffer,
    // Logic
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    pipeline: Pipeline,
    // Descriptors
    uniform_pool: DescriptorPool,
    mvp_set: DescriptorSet,
    // Commands
    graphics_pool: CommandPool,
    transfer_pool: CommandPool,
    draw: CommandBuffer,
    transfer: CommandBuffer,
    // Syncs
    transfer_done: Semaphore,
}

impl Renderer {
    pub fn new(device: &Device, swapchain_images: &Vec<Image>) -> Renderer {
        // Queues
        let graphics_queue = unsafe { device.get_device_queue(device.infos.graphics_idx, 0) };
        let transfer_queue = unsafe { device.get_device_queue(device.infos.transfer_idx, 0) };

        // Resources
        let swapchain_image_views =
            resources::create_swapchain_image_views(device, swapchain_images);
        let vertices = resources::allocate_vertices(device);
        let staging_vertices = resources::allocate_staging_vertices(device);
        let indices = resources::allocate_indices(device);
        let staging_indices = resources::allocate_staging_indices(device);

        // Logic
        let render_pass = RenderPass::new(device);
        let framebuffers = create_framebuffers(device, &render_pass, &swapchain_image_views);
        let pipeline = Pipeline::new(device, &render_pass);

        // Descriptors
        let mvp = resources::allocate_mvp(device);
        let uniform_pool = descriptors::create_uniform_buffer_pool(device);
        let set_layouts = [*pipeline.mvp_layout()];
        let mvp_set =
            descriptors::allocate_configure_mvp_set(device, &uniform_pool, &set_layouts, &mvp);

        // Commands
        let graphics_pool = commands::create_graphics_pool(device);
        let transfer_pool = commands::create_transfer_pool(device);
        let draw = commands::allocate_draw(device, graphics_pool);
        let transfer = commands::allocate_record_transfer(
            device,
            transfer_pool,
            &staging_vertices,
            &vertices,
            &staging_indices,
            &indices,
        );

        // Syncs
        let transfer_done = device.bp_new_semaphore();

        Renderer {
            graphics_queue,
            transfer_queue,
            swapchain_image_views,
            vertices,
            staging_vertices,
            indices,
            staging_indices,
            mvp,
            uniform_pool,
            mvp_set,
            render_pass,
            framebuffers,
            pipeline,
            graphics_pool,
            transfer_pool,
            draw,
            transfer,
            transfer_done,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            // Syncs
            device.destroy_semaphore(self.transfer_done, None);

            // Commands
            device.destroy_command_pool(self.graphics_pool, None);
            device.destroy_command_pool(self.transfer_pool, None);

            // Descriptors
            device.destroy_descriptor_pool(self.uniform_pool, None);

            // Resources
            for image_view in &mut self.swapchain_image_views {
                device.destroy_image_view(*image_view, None);
            }
            device.ct_destroy_buffer(&mut self.vertices);
            device.ct_destroy_buffer(&mut self.staging_vertices);
            device.ct_destroy_buffer(&mut self.indices);
            device.ct_destroy_buffer(&mut self.staging_indices);
            device.ct_destroy_mapped_buffer(&mut self.mvp);

            // Logic
            for framebuffer in &mut self.framebuffers {
                device.destroy_framebuffer(*framebuffer, None);
            }
            self.pipeline.destroy(device);
            device.destroy_render_pass(*self.render_pass, None);
        }
    }

    pub fn submit_render(
        &mut self,
        device: &Device,
        meshes: Vec<(Mat4, Mesh)>,
        camera: &Camera,
        swapchain_image_idx: u32,
        swapchain_extent: Extent2D,
        image_available: Semaphore,
        rendering_done: Semaphore,
        fence_rendering_done: Fence,
    ) {
        // CPU COPY : staging vertices
        self.copy_vertices(device, &meshes);

        // SUBMIT : transfer
        let signal_semaphores = [self.transfer_done];
        self.submit_transfer(device, &signal_semaphores);

        // CPU COPY : mvp
        self.copy_mvp(camera, swapchain_extent, &meshes);

        // RECORD : draw
        self.record_draw(device, swapchain_image_idx as usize);

        // SUBMIT : draw
        let wait_semaphores = [self.transfer_done, image_available];
        let wait_dst_stage_mask = [
            PipelineStageFlags::VERTEX_INPUT,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ];
        let signal_semaphores = [rendering_done];
        let signal_fence = fence_rendering_done;
        self.submit_draw(
            device,
            &wait_semaphores,
            &wait_dst_stage_mask,
            &signal_semaphores,
            signal_fence,
        );
    }

    fn copy_vertices(&mut self, device: &Device, meshes: &Vec<(Mat4, Mesh)>) {
        let vertices = &meshes[0].1.vertices;
        let indices = &meshes[0].1.indices;
        unsafe {
            // map
            let mapped_vertices = device
                .allocator()
                .map_memory(&mut self.staging_vertices.allocation)
                .expect("Failed to map memory.");
            let mapped_indices = device
                .allocator()
                .map_memory(&mut self.staging_indices.allocation)
                .expect("Failed to map memory.");

            // copy
            mapped_vertices.copy_from(
                vertices.as_ptr() as *const u8,
                Vertex::size_of() * vertices.len(),
            );
            mapped_indices.copy_from(
                indices.as_ptr() as *const u8,
                size_of::<u32>() * indices.len(),
            );

            // unmap
            device
                .allocator()
                .unmap_memory(&mut self.staging_vertices.allocation);
            device
                .allocator()
                .unmap_memory(&mut self.staging_indices.allocation);
        }
    }

    fn copy_mvp(&mut self, camera: &Camera, extent: Extent2D, meshes: &Vec<(Mat4, Mesh)>) {
        let transform = meshes[0].0;
        let mvp = MVP::from_camera_transform(camera, extent, transform);
        let ptr: *const MVP = &mvp;
        unsafe { self.mvp.ptr.copy_from(ptr as *const u8, MVP::size_of()) };
    }

    fn submit_transfer(&self, device: &Device, signal_semaphores: &[Semaphore]) {
        let command_buffers = [self.transfer];
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe {
            device
                .queue_submit(self.transfer_queue, &[submit_info], Fence::null())
                .expect("Failed to submit transfer.");
        }
    }

    fn submit_draw(
        &self,
        device: &Device,
        wait_semaphores: &[Semaphore],
        wait_dst_stage_mask: &[PipelineStageFlags],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffers = [self.draw];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .signal_semaphores(&signal_semaphores)
            .command_buffers(&command_buffers);
        unsafe {
            device
                .queue_submit(self.graphics_queue, &[submit_info], signal_fence)
                .expect("Failed to submit draw.");
        }
    }
}
