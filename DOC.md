# Custom devices

`ash::Device` are wrapped into my own Device objects. It allows to :
- Keep accessible physical device informations (queue family indices, surface capabilities, etc.).
- Easily provide vulkan extension fns (like the ones in `ash::khr::swapchain::Device`).
- Provide memory-management fns with a dedicated VMA allocator.
- Provide boilerplate-free fns.

The design is setup for now as if each "engines" have its own device (can share the same physical device though).