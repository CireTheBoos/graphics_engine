mod device;
mod presenter;
mod renderer;

use crate::{
    boilerplate::{new_fence, new_semaphore, wait_reset_fences},
    instance::Instance,
    model::Vertex,
};
use ash::vk::{Fence, Semaphore, SurfaceKHR};
pub use device::Device;
pub use presenter::Presenter;
pub use renderer::Renderer;

const FLIGHTS: usize = 2;

// Given a surface :
// - Renders imgs from vertices
// - Presents them
pub struct GraphicsEngine {
    // Essentials
    surface: SurfaceKHR,
    device: Device,
    // Missions
    presenter: Presenter,
    renderer: Renderer,
    // Sync
    img_available: Semaphore,
    render_finished: Semaphore,
    presented: Fence,
}

impl GraphicsEngine {
    pub fn new(instance: &Instance, surface: SurfaceKHR) -> GraphicsEngine {
        // Essentials
        let device = Device::new(instance, &surface);

        // Missions
        let presenter = Presenter::new(&device, &surface);
        let renderer = Renderer::new(&device, &presenter);

        // Sync
        let img_available = new_semaphore(&device);
        let render_finished = new_semaphore(&device);
        let presented = new_fence(&device, true);

        GraphicsEngine {
            surface,
            device,
            presenter,
            renderer,
            img_available,
            render_finished,
            presented,
        }
    }

    // Destroy vulkan objects (order matters)
    pub fn destroy(&mut self, instance: &Instance) {
        unsafe {
            // wait unfinished work
            self.device.device_wait_idle().unwrap();
            // destroy syncs
            self.device.destroy_semaphore(self.img_available, None);
            self.device.destroy_semaphore(self.render_finished, None);
            self.device.destroy_fence(self.presented, None);
            // destroy missions
            self.presenter.destroy(&self.device);
            self.renderer.destroy(&self.device, self.device.allocator());
            // destroy surface
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn frame(&mut self, vertices: &Vec<Vertex>) {
        // WAIT
        let fences = [self.presented];
        wait_reset_fences(&self.device, &fences, false, None);

        // Acquire next image
        let signal_semaphore = self.img_available;
        let signal_fence = Fence::null();
        let image_idx =
            self.presenter
                .acquire_next_image(&self.device, signal_semaphore, signal_fence);

        // submit rendering
        let wait_semaphores = [self.img_available];
        let signal_semaphores = [self.render_finished];
        let signal_fence = self.presented;
        self.renderer.submit_render(
            &self.device,
            vertices,
            &wait_semaphores,
            &signal_semaphores,
            signal_fence,
        );

        //submit present
        let wait_semaphores = [self.render_finished];
        self.presenter
            .present(&self.device, image_idx, &wait_semaphores);
    }
}
