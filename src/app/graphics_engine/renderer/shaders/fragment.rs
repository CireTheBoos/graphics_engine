use ash::vk::ShaderModule;
use shaderc::ShaderKind;

use crate::app::graphics_engine::Device;

use super::compiler::Compiler;

pub const FRAGMENT: &str = " 
#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
";

impl Compiler {
    pub fn fragment(&self, device: &Device) -> ShaderModule {
        self.to_shader_module(device, FRAGMENT, ShaderKind::Fragment, "fragment.glsl")
    }
}
