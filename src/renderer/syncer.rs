use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use super::Device;

// Translates boilerplate sync code into meaningful fns for renderer
pub struct Syncer {
    current_frame: usize,
    pub frames: Vec<Frame>,
    pub last_frame_transfer_done: Fence,
}

pub struct Frame {
    pub idx: usize,
    // in-frame dependencies
    pub transfer_done: Semaphore,
    pub img_available: Semaphore,
    pub render_finished: Semaphore,
    // between last flight dependencies
    pub last_flight_presented: Fence,
}

impl Syncer {
    pub fn new(device: &Device) -> Syncer {
        let mut frames = Vec::new();
        for idx in 0..super::FRAMES_IN_FLIGHT {
            // transfer_done
            let semaphore_create_info = SemaphoreCreateInfo::default();
            let transfer_done = unsafe { device.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore.");

            // img_available
            let semaphore_create_info = SemaphoreCreateInfo::default();
            let img_available = unsafe { device.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore.");

            // render_finished
            let semaphore_create_info = SemaphoreCreateInfo::default();
            let render_finished = unsafe { device.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore.");

            // last_flight_presented
            let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
            let last_flight_presented = unsafe { device.create_fence(&fence_create_info, None) }
                .expect("Failed to create fence.");

            frames.push(Frame {
                idx,
                transfer_done,
                img_available,
                render_finished,
                last_flight_presented,
            });
        }

        // last_frame_transfer_done
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let last_frame_transfer_done = unsafe { device.create_fence(&fence_create_info, None) }
            .expect("Failed to create fence.");

        Syncer {
            current_frame: 0,
            frames,
            last_frame_transfer_done,
        }
    }

    pub fn step(&mut self) {
        self.current_frame = (self.current_frame + 1) % super::FRAMES_IN_FLIGHT;
    }

    pub fn current_frame(&self) -> &Frame {
        &self.frames[self.current_frame]
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            for frame in &self.frames {
                device.destroy_semaphore(frame.img_available, None);
                device.destroy_semaphore(frame.render_finished, None);
                device.destroy_fence(frame.last_flight_presented, None);
                device.destroy_semaphore(frame.transfer_done, None);
            }
            device.destroy_fence(self.last_frame_transfer_done, None);
        }
    }

    pub fn wait_fences(&self, device: &Device, fences: &[Fence]) {
        unsafe {
            device
                .wait_for_fences(&fences, true, u64::MAX)
                .expect("Failed to wait on previous frame.");
            device
                .reset_fences(&fences)
                .expect("Failed to reset fence.");
        }
    }
}
