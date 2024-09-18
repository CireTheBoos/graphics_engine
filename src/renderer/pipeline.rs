use std::ptr::NonNull;

use super::{device::RendererDevice, shaders::ShaderManager};

pub struct RendererPipeline {
    device: NonNull<RendererDevice>,
}

impl RendererPipeline {
    pub fn new(device: &RendererDevice) -> RendererPipeline {
        // compiling shaders
        let shader_manager = ShaderManager::new(device);
        let vertex = shader_manager.vertex();
        let fragment = shader_manager.fragment();

        // Create pipeline

        // Cleanup and return
        unsafe { device.destroy_shader_module(vertex, None) };
        unsafe { device.destroy_shader_module(fragment, None) };
        RendererPipeline {
            device: NonNull::from(device),
        }
    }
}
