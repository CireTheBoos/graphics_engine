use ash::vk::PhysicalDevice;
use std::ops::Deref;
use vk_mem::{Allocation, Allocator, AllocatorCreateInfo};

use crate::app::instance::Instance;

// For now will have the name CustomBuffer until it proved it's useful
pub struct CustomBuffer {
    pub buffer: ash::vk::Buffer,
    pub allocation: Allocation,
}

impl Deref for CustomBuffer {
    type Target = ash::vk::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl CustomBuffer {
    pub fn destroy(&mut self, allocator: &Allocator) {
        unsafe { allocator.destroy_buffer(self.buffer, &mut self.allocation) };
    }
}

pub struct CustomMappedBuffer {
    pub buffer: CustomBuffer,
    pub ptr: *mut u8,
}

impl CustomMappedBuffer {
    pub fn new(allocator: &Allocator, mut buffer: CustomBuffer) -> CustomMappedBuffer {
        // Map
        let ptr = unsafe {
            allocator
                .map_memory(&mut buffer.allocation)
                .expect("Failed to map memory.")
        };
        CustomMappedBuffer { buffer, ptr }
    }

    pub fn destroy(&mut self, allocator: &Allocator) {
        // Unmap
        unsafe {
            allocator.unmap_memory(&mut self.buffer.allocation);
        }
        self.buffer.destroy(allocator);
    }
}

pub fn create_allocator(
    instance: &Instance,
    device: &ash::Device,
    physical_device: PhysicalDevice,
) -> Allocator {
    let create_info = AllocatorCreateInfo::new(instance, device, physical_device);
    unsafe { Allocator::new(create_info).expect("Failed to create allocator.") }
}
