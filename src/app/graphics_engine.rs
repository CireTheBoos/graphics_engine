mod device;
mod mesher;
mod presenter;
mod renderer;

use crate::app::{instance::Instance, model::Camera};
use ash::vk::{Fence, Semaphore, SurfaceKHR};
pub use device::Device;
pub use mesher::ToMesh;
pub use presenter::Presenter;
pub use renderer::Renderer;

// Given a surfaceKHR :
// - Creates meshes from objects = mesher (hold no data)
// - Renders imgs from meshes = renderer
// - Presents imgs = presenter
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

        // Syncs
        let image_available = device.bp_new_semaphore();
        let rendering_done = device.bp_new_semaphore();
        let fence_rendering_done = device.bp_new_fence(true);

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
            self.renderer.destroy(&self.device);
            // destroy surface
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn frame(&mut self, objects: Vec<&dyn ToMesh>, camera: &Camera) {
        // Wait last rendering
        self.device
            .bp_wait_reset_fence(self.fence_rendering_done, None);

        // Acquire next image
        let (image_idx, _) = self
            .presenter
            .acquire_next_image(&self.device, self.image_available);

        // Translates objects into meshes
        let meshes = objects
            .into_iter()
            .map(|object| (object.transform(), object.mesh()))
            .collect();

        // Render to it
        self.renderer.submit_render(
            &self.device,
            meshes,
            camera,
            image_idx,
            self.device.infos.capabilities.current_extent,
            self.image_available,
            self.rendering_done,
            self.fence_rendering_done,
        );

        // Present it
        self.presenter
            .present(&self.device, image_idx, self.rendering_done);
    }
}
