use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use super::{Device, FLIGHTS};

// Translates boilerplate sync code into meaningful fns for renderer
#[derive(Debug)]
pub struct Syncer {
    pub transfer_done: Fence,
    //pub img_acquired: Fence,
    pub flights: Vec<Flight>,
    current_flight: usize,
}

#[derive(Debug)]
pub struct Flight {
    pub idx: usize,
    // in-frame dependencies
    pub transfer_done: Semaphore,
    pub img_available: Semaphore,
    pub rendering_done: Semaphore,
    // between-flight dependencies
    pub presented: Fence,
}

impl Syncer {
    pub fn new(device: &Device) -> Syncer {
        let mut flights = Vec::with_capacity(FLIGHTS);
        for idx in 0..FLIGHTS {
            // Semaphores
            let transfer_done = new_semaphore(device);
            let img_available = new_semaphore(device);
            let rendering_done = new_semaphore(device);

            // Fences
            let presented = new_fence(device, true);

            flights.push(Flight {
                idx,
                transfer_done,
                img_available,
                rendering_done,
                presented,
            });
        }
        let transfer_done = new_fence(device, true);
        //let img_acquired = new_fence(device, true);
        Syncer {
            transfer_done,
            //img_acquired,
            flights,
            current_flight: 0,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device.destroy_fence(self.transfer_done, None);
            //device.destroy_fence(self.img_acquired, None);
            for flight in &self.flights {
                device.destroy_semaphore(flight.img_available, None);
                device.destroy_semaphore(flight.rendering_done, None);
                device.destroy_semaphore(flight.transfer_done, None);
                device.destroy_fence(flight.presented, None);
            }
        }
    }

    pub fn step_flight(&mut self) {
        self.current_flight = (self.current_flight + 1) % FLIGHTS;
    }

    pub fn current_flight(&self) -> &Flight {
        &self.flights[self.current_flight]
    }
}

pub fn wait_fences(device: &Device, fences: &[Fence]) {
    unsafe {
        device
            .wait_for_fences(&fences, true, u64::MAX)
            .expect("Failed to wait on previous frame.");
        device
            .reset_fences(&fences)
            .expect("Failed to reset fence.");
    }
}

fn new_semaphore(device: &Device) -> Semaphore {
    let semaphore_create_info = SemaphoreCreateInfo::default();
    unsafe { device.create_semaphore(&semaphore_create_info, None) }
        .expect("Failed to create semaphore.")
}

fn new_fence(device: &Device, signaled: bool) -> Fence {
    let fence_create_info = if signaled {
        FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED)
    }
    else {
        FenceCreateInfo::default()
    };
    unsafe { device.create_fence(&fence_create_info, None) }
        .expect("Failed to create fence.")
}