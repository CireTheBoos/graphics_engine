use ash::vk::{ShaderModule, ShaderModuleCreateInfo};
use shaderc::{Compiler, ShaderKind};

use super::Device;

const VERTEX_SHADER_SRC: &str = "
#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
}
";

const FRAGMENT_SHADER_SRC: &str = " 
#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
";

pub struct ShaderManager<'a> {
    compiler: Compiler,
    device: &'a Device,
}

impl ShaderManager<'_> {
    pub fn new<'a>(device: &'a Device) -> ShaderManager<'a> {
        let compiler = Compiler::new().unwrap();
        ShaderManager { compiler, device }
    }

    pub fn vertex(&self) -> ShaderModule {
        // Compile
        let binary_result = self
            .compiler
            .compile_into_spirv(
                VERTEX_SHADER_SRC,
                ShaderKind::Vertex,
                "vertex.glsl",
                "main",
                None,
            )
            .unwrap();
        let code = binary_result.as_binary();

        // Create shader module
        let create_info = ShaderModuleCreateInfo::default().code(code);
        unsafe { self.device.create_shader_module(&create_info, None) }
            .expect("Failed to create shader module")
    }

    pub fn fragment(&self) -> ShaderModule {
        // Compile
        let binary_result = self
            .compiler
            .compile_into_spirv(
                FRAGMENT_SHADER_SRC,
                ShaderKind::Fragment,
                "frafment.glsl",
                "main",
                None,
            )
            .unwrap();

        let code = binary_result.as_binary();

        // Create shader module
        let create_info = ShaderModuleCreateInfo::default().code(code);
        unsafe { self.device.create_shader_module(&create_info, None) }
            .expect("Failed to create shader module")
    }
}
