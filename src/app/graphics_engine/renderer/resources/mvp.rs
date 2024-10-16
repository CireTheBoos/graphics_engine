use std::f32::consts::FRAC_PI_4;

use ash::vk::{BufferCreateInfo, BufferUsageFlags, Extent2D, MemoryPropertyFlags, SharingMode};
use glam::Mat4;
use vk_mem::AllocationCreateInfo;

use crate::app::{
    graphics_engine::{device::MappedBuffer, Device},
    model::Camera,
};

#[repr(C)]
pub struct MVP {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

impl MVP {
    pub fn from_camera_transform(camera: &Camera, extent: Extent2D, transform: Mat4) -> MVP {
        let model = transform;
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

pub fn allocate_mvp(device: &Device) -> MappedBuffer {
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

    device.ct_create_mapped_buffer(&buffer_info, &create_info)
}
