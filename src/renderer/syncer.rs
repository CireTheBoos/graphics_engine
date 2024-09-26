use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use super::Device;

pub const FRAMES_IN_FLIGHT: usize = 2;

// Translates boilerplate sync code  into meaningful fns for renderer
pub struct Syncer {
    pub current_frame: usize,
    pub frames: Vec<FrameSyncs>
}

pub struct FrameSyncs {
    pub img_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight: Fence,
}

impl Syncer {
    pub fn new(device: &Device) -> Syncer {
        let mut frames = Vec::new();
        for _ in 0..FRAMES_IN_FLIGHT {
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

            frames.push(FrameSyncs {
                img_available,
                render_finished,
                in_flight,
            });
        }
        Syncer {
            current_frame: 0,
            frames,
        }
    }

    pub fn next_frame(&self) -> usize {
        (self.current_frame + 1) % FRAMES_IN_FLIGHT
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            for frame in &self.frames {
                device.destroy_semaphore(frame.img_available, None);
                device.destroy_semaphore(frame.render_finished, None);
                device.destroy_fence(frame.in_flight, None);
            }
            
        }
    }

    pub fn wait_in_flight(&self, device: &Device, frame: usize) {
        unsafe {
            device
                .wait_for_fences(&[self.frames[frame].in_flight], true, u64::MAX)
                .expect("Failed to wait on previous frame.");
            device
                .reset_fences(&[self.frames[frame].in_flight])
                .expect("Failed to reset fence.");
        }
    }
}
