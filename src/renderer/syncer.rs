use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use super::Device;

// Translates boilerplate sync code  nto meaningful fns for renderer
pub struct Syncer {
    current_frame: usize,
    pub frames: Vec<Frame>,
}

pub struct Frame {
    pub idx: usize,
    pub img_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight: Fence,
}

impl Syncer {
    pub fn new(device: &Device) -> Syncer {
        let mut frames = Vec::new();
        for idx in 0..super::FRAMES_IN_FLIGHT {
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

            frames.push(Frame {
                idx,
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
                device.destroy_fence(frame.in_flight, None);
            }
        }
    }

    pub fn wait_in_flight(&self, device: &Device, frame: &Frame) {
        unsafe {
            device
                .wait_for_fences(&[frame.in_flight], true, u64::MAX)
                .expect("Failed to wait on previous frame.");
            device
                .reset_fences(&[frame.in_flight])
                .expect("Failed to reset fence.");
        }
    }
}
