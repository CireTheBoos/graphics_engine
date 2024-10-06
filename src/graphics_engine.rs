mod device;
mod presenter;
mod renderer;

use crate::{instance::Instance, model::Vertex};
use ash::vk::{Fence, PipelineStageFlags, Semaphore, SurfaceKHR};
pub use device::Device;
pub use presenter::Presenter;
pub use renderer::Renderer;
use vk_mem::Allocator;

use crate::sync;

const FLIGHTS: usize = 2;

// Given a surface :
// - Renders imgs from vertices
// - Presents them
pub struct GraphicsEngine {
    // Essentials ( /!\ allocator must drop before device )
    allocator: Allocator,
    surface: SurfaceKHR,
    device: Device,
    // Missions
    presenter: Presenter,
    renderer: Renderer,
    // Sync
    img_available: Semaphore,
    transfer_done: Semaphore,
    render_finished: Semaphore,
    presented: Fence,
}

impl GraphicsEngine {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> GraphicsEngine {
        // Essentials
        let device = Device::new(instance, &surface);
        let allocator = crate::allocator::new(instance, &device, device.infos.physical_device);

        // Missions
        let presenter = Presenter::new(&device, &surface);
        let renderer = Renderer::new(&device, &presenter, &allocator);

        // Sync
        let img_available = sync::new_semaphore(&device);
        let transfer_done = sync::new_semaphore(&device);
        let render_finished = sync::new_semaphore(&device);
        let presented = sync::new_fence(&device, true);

        GraphicsEngine {
            surface,
            device,
            allocator,
            presenter,
            renderer,
            img_available,
            transfer_done,
            render_finished,
            presented,
        }
    }

    // Destroy vulkan objects (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.device.destroy_semaphore(self.img_available, None);
            self.device.destroy_semaphore(self.render_finished, None);
            self.device.destroy_semaphore(self.transfer_done, None);
            self.device.destroy_fence(self.presented, None);
            self.presenter.destroy(&self.device);
            self.renderer.destroy(&self.device, &self.allocator);
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn frame(&mut self, vertices: &Vec<Vertex>) {
        // WAIT
        let fences = [self.presented];
        sync::wait_fences(&self.device, &fences, false, None);

        // Update staging vertex buffer
        self.renderer.copy_vertices(vertices, &self.allocator);

        // SUBMIT : Transfer
        let signal_semaphores = [self.transfer_done];
        let signal_fence = Fence::null();
        self.renderer
            .transfer(&self.device, &signal_semaphores, signal_fence);

        // Acquire next image
        let signal_semaphore = self.img_available;
        let signal_fence = Fence::null();
        let image_idx =
            self.presenter
                .acquire_next_image(&self.device, signal_semaphore, signal_fence);

        // RECORD : draw
        self.renderer.record_draw(&self.device, 0);

        // SUBMIT : draw
        let wait_semaphores = [self.img_available, self.transfer_done];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished];
        let signal_fence = self.presented;
        self.renderer.draw(
            &self.device,
            &wait_semaphores,
            &wait_dst_stage_mask,
            &signal_semaphores,
            signal_fence,
        );

        // PRESENT
        let wait_semaphores = [self.render_finished];
        self.presenter
            .present(&self.device, image_idx, &wait_semaphores);
    }
}
