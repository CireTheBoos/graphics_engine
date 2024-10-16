use ash::vk::BufferCreateInfo;
use std::ops::Deref;
use vk_mem::{Alloc, Allocation, AllocationCreateInfo};

use super::Device;

pub struct Buffer {
    pub buffer: ash::vk::Buffer,
    pub allocation: Allocation,
}

// Deref : ash::vk::Buffer
impl Deref for Buffer {
    type Target = ash::vk::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

pub struct MappedBuffer {
    pub buffer: ash::vk::Buffer,
    pub allocation: Allocation,
    pub ptr: *mut u8,
}

// Deref : ash::vk::Buffer
impl Deref for MappedBuffer {
    type Target = ash::vk::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Device {
    pub fn ct_create_buffer(
        &self,
        buffer_info: &BufferCreateInfo,
        create_info: &AllocationCreateInfo,
    ) -> Buffer {
        let (buffer, allocation) = unsafe {
            self.allocator()
                .create_buffer(buffer_info, create_info)
                .expect("Failed to create vertex buffer.")
        };
        Buffer { buffer, allocation }
    }

    pub fn ct_destroy_buffer(&self, buffer: &mut Buffer) {
        unsafe { self.allocator().destroy_buffer(buffer.buffer, &mut buffer.allocation) };
    }

    pub fn ct_create_mapped_buffer(
        &self,
        buffer_info: &BufferCreateInfo,
        create_info: &AllocationCreateInfo,
    ) -> MappedBuffer {
        let (buffer, mut allocation) = unsafe {
            self.allocator()
                .create_buffer(buffer_info, create_info)
                .expect("Failed to create vertex buffer.")
        };
        let ptr = unsafe {
            self.allocator()
                .map_memory(&mut allocation)
                .expect("Failed to map memory")
        };
        MappedBuffer {
            buffer,
            allocation,
            ptr,
        }
    }

    pub fn ct_destroy_mapped_buffer(&self, mapped_buffer: &mut MappedBuffer) {
        unsafe {
            self.allocator().unmap_memory(&mut mapped_buffer.allocation);
            self.allocator().destroy_buffer(mapped_buffer.buffer, &mut mapped_buffer.allocation);
        }
    }
}
