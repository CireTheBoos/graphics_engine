mod commander;
mod dealer;
mod device;
mod presenter;
mod renderer;
mod syncer;

use crate::{instance::Instance, model::Vertex};
use ash::vk::{Fence, PipelineStageFlags, SurfaceKHR};
pub use commander::Commander;
pub use dealer::Dealer;
pub use device::Device;
pub use presenter::Presenter;
pub use renderer::Renderer;
pub use syncer::Syncer;

const FLIGHTS: usize = 2;

// Given a surface :
// - Render imgs from vertices
// - Presents them
pub struct GraphicsEngine {
    // Support
    commander: Commander,
    syncer: Syncer,
    dealer: Dealer,
    // Assistants
    presenter: Presenter,
    renderer: Renderer,
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
        let renderer = Renderer::new(&device, &presenter);

        GraphicsEngine {
            surface,
            device,
            commander,
            syncer,
            dealer,
            presenter,
            renderer,
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
            self.renderer.destroy(&self.device);

            // Essentials
            instance.surface_khr().destroy_surface(self.surface, None);
        }
    }

    pub fn render_frame(&mut self, vertices: &Vec<Vertex>) {
        println!("{:#?}", self.syncer);
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
        self.renderer.transfer_vertices(
            &self.device,
            &self.commander,
            &signal_semaphores,
            signal_fence,
        );

        // Acquire next image
        let signal_semaphore = img_available;
        let signal_fence = Fence::null();
        let image_idx =
            self.presenter
                .acquire_next_image(&self.device, signal_semaphore, signal_fence);

        // RECORD : draw
        self.renderer
            .record_draw(&self.device, &self.syncer, &self.dealer, &self.commander);

        // SUBMIT : draw
        let wait_semaphores = [img_available, transfer_done];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [rendering_done];
        let signal_fence = presented;
        self.renderer.draw(
            &self.device,
            &self.commander,
            &self.syncer,
            &wait_semaphores,
            &wait_dst_stage_mask,
            &signal_semaphores,
            signal_fence,
        );

        // PRESENT
        let wait_semaphores = [rendering_done];
        self.presenter
            .present(&self.device, image_idx, &wait_semaphores);

        self.syncer.step_flight();
    }
}
