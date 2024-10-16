use ash::vk::ShaderModule;
use shaderc::ShaderKind;

use crate::app::graphics_engine::Device;

use super::compiler::Compiler;

pub const VERTEX: &str = "
#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 1.0);
    fragColor = inColor;
}
";

impl Compiler {
    pub fn vertex(&self, device: &Device) -> ShaderModule {
        self.to_shader_module(device, VERTEX, ShaderKind::Vertex, "vertex.glsl")
    }
}
