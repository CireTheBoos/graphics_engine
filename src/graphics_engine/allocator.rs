use std::ops::Deref;

use ash::{vk::PhysicalDevice, Device};
use vk_mem::{Allocation, Allocator, AllocatorCreateInfo};

use crate::instance::Instance;

pub struct Buffer {
    pub buffer: ash::vk::Buffer,
    pub allocation: Allocation,
}

impl Deref for Buffer {
    type Target = ash::vk::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Buffer {
    pub fn destroy(&mut self, allocator: &Allocator) {
        unsafe { allocator.destroy_buffer(self.buffer, &mut self.allocation) };
    }
}

pub fn new(instance: &Instance, device: &Device, physical_device: PhysicalDevice) -> Allocator {
    let create_info = AllocatorCreateInfo::new(instance, device, physical_device);
    unsafe { Allocator::new(create_info) }.expect("Failed to create allocator.")
}
