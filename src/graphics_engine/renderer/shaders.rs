mod fragment;
mod vertex;

use std::ops::Deref;

use ash::vk::{ShaderModule, ShaderModuleCreateInfo};
use shaderc::ShaderKind;

use crate::graphics_engine::Device;

pub struct Compiler {
    compiler: shaderc::Compiler,
}

impl Deref for Compiler {
    type Target = shaderc::Compiler;
    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

impl Compiler {
    pub fn new() -> Compiler {
        let compiler = shaderc::Compiler::new().unwrap();
        Compiler { compiler }
    }

    pub fn vertex(&self, device: &Device) -> ShaderModule {
        self.to_shader_module(device, vertex::VERTEX, ShaderKind::Vertex, "vertex.glsl")
    }

    pub fn fragment(&self, device: &Device) -> ShaderModule {
        self.to_shader_module(
            device,
            fragment::FRAGMENT,
            ShaderKind::Fragment,
            "fragment.glsl",
        )
    }

    fn to_shader_module(
        &self,
        device: &Device,
        source_text: &str,
        shader_kind: ShaderKind,
        input_file_name: &str,
    ) -> ShaderModule {
        // Compile
        let binary_result = self
            .compile_into_spirv(source_text, shader_kind, input_file_name, "main", None)
            .unwrap();
        let code = binary_result.as_binary(); // points to binary_result

        // Create shader module
        let create_info = ShaderModuleCreateInfo::default().code(code);
        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }
}
