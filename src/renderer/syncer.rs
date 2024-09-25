use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use super::Device;

// Translates boilerplate sync code  into meaningful fns for renderer
pub struct Syncer {
    pub img_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight: Fence,
}

impl Syncer {
    pub fn new(device: &Device) -> Syncer {
        // img_available
        let semaphore_create_info = SemaphoreCreateInfo::default();
        let img_available = unsafe { device.create_semaphore(&semaphore_create_info, None) }
            .expect("Failed to create semaphore.");

        // render_finished
        let semaphore_create_info = SemaphoreCreateInfo::default();
        let render_finished = unsafe { device.create_semaphore(&semaphore_create_info, None) }
            .expect("Failed to create semaphore.");

        // in_flight
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let in_flight = unsafe { device.create_fence(&fence_create_info, None) }
            .expect("Failed to create fence.");

        Syncer {
            img_available,
            render_finished,
            in_flight,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.img_available, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_fence(self.in_flight, None);
        }
    }

    pub fn wait_in_flight(&self, device: &Device) {
        unsafe {
            device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .expect("Failed to wait on previous frame.");
            device
                .reset_fences(&[self.in_flight])
                .expect("Failed to reset fence.");
        }
    }
}
