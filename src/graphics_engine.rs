mod device;
mod presenter;
mod renderer;

use crate::{
    boilerplate::{new_fence, new_semaphore, wait_reset_fence},
    instance::Instance,
    model::Vertex,
};
use ash::vk::{Fence, Semaphore, SurfaceKHR};
pub use device::Device;
pub use presenter::Presenter;
pub use renderer::Renderer;

// Given a surface :
// - Renders to imgs from vertices
// - Presents them
pub struct GraphicsEngine {
    // Essentials
    surface: SurfaceKHR,
    device: Device,
    // Missions
    presenter: Presenter,
    renderer: Renderer,
    // Syncs
    image_available: Semaphore,
    rendering_done: Semaphore,
    fence_rendering_done: Fence,
}

impl GraphicsEngine {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> GraphicsEngine {
        // Essentials
        let device = Device::new(instance, &surface);

        // Missions
        let presenter = Presenter::new(&device, &surface);
        let renderer = Renderer::new(&device, presenter.swapchain_images());

        // Sync
        let image_available = new_semaphore(&device);
        let rendering_done = new_semaphore(&device);
        let fence_rendering_done = new_fence(&device, true);

        GraphicsEngine {
            surface,
            device,
            presenter,
            renderer,
            image_available,
            rendering_done,
            fence_rendering_done,
        }
    }

    // Destroy vulkan objects (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            // wait unfinished work
            self.device.device_wait_idle().unwrap();
            // destroy syncs
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_semaphore(self.rendering_done, None);
            self.device.destroy_fence(self.fence_rendering_done, None);
            // destroy missions
            self.presenter.destroy(&self.device);
            self.renderer.destroy(&self.device, self.device.allocator());
            // destroy surface
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn frame(&mut self, vertices: &Vec<Vertex>, indices: &Vec<u32>) {
        // Wait last rendering
        wait_reset_fence(&self.device, self.fence_rendering_done, None);

        // Acquire next image
        let (image_idx, _) = self
            .presenter
            .acquire_next_image(&self.device, self.image_available);

        // Render to it
        self.renderer.submit_render(
            &self.device,
            vertices,
            indices,
            image_idx,
            self.image_available,
            self.rendering_done,
            self.fence_rendering_done,
        );

        // Present it
        self.presenter
            .present(&self.device, image_idx, self.rendering_done);
    }
}
