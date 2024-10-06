use ash::vk::PhysicalDevice;
use std::ops::Deref;
use vk_mem::{Allocation, Allocator, AllocatorCreateInfo};

use crate::instance::Instance;

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

pub fn create_allocator(
    instance: &Instance,
    device: &ash::Device,
    physical_device: PhysicalDevice,
) -> Allocator {
    let create_info = AllocatorCreateInfo::new(instance, device, physical_device);
    unsafe { Allocator::new(create_info) }.expect("Failed to create allocator.")
}
