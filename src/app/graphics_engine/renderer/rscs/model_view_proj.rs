use std::f32::consts::FRAC_PI_4;

use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, DescriptorBufferInfo, DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorType, Extent2D, MemoryPropertyFlags, SharingMode, WriteDescriptorSet, WHOLE_SIZE
};
use glam::Mat4;
use vk_mem::AllocationCreateInfo;

use crate::app::{
    graphics_engine::{device::CustomMappedBuffer, Device},
    model::Camera,
};

#[repr(C)]
pub struct MVP {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

impl MVP {
    pub fn from_camera(camera: &Camera, extent: Extent2D) -> MVP {
        let model = glam::Mat4::IDENTITY;
        let view = glam::Mat4::look_at_rh(camera.eye, camera.center, camera.up);
        let fov_y_radians = FRAC_PI_4;
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let proj = glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, 0.1, 100.);

        MVP { model, view, proj }
    }
    pub fn size_of() -> usize {
        size_of::<MVP>()
    }
}

pub fn allocate_mvp(device: &Device) -> CustomMappedBuffer {
    let size = MVP::size_of() as u64;
    let queue_family_indices = [device.infos.graphics_idx];
    let buffer_info = BufferCreateInfo::default()
        .queue_family_indices(&queue_family_indices)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .size(size)
        .usage(BufferUsageFlags::UNIFORM_BUFFER);

    let create_info = AllocationCreateInfo {
        required_flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        ..Default::default()
    };

    let buffer = super::create_buffer(device, &buffer_info, &create_info);

    CustomMappedBuffer::new(device.allocator(), buffer)
}

pub fn allocate_configure_mvp_set(
    device: &Device,
    descriptor_pool: &DescriptorPool,
    set_layouts: &[DescriptorSetLayout],
    buffer: &Buffer,
) -> DescriptorSet {
    let set = super::allocate_descriptor_sets(device, descriptor_pool, set_layouts)[0];
    let buffer_info = DescriptorBufferInfo::default()
        .buffer(*buffer)
        .offset(0)
        .range(WHOLE_SIZE);
    let buffer_infos = [buffer_info];
    let write = WriteDescriptorSet::default()
        .buffer_info(&buffer_infos)
        .dst_set(set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1);
    let descriptor_writes = [write];

    unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    set
}
