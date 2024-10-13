- [Structure](#structure)
    - [Graphics engine](#graphics-engine)
- [Custom vulkan devices](#custom-vulkan-devices)


# Structure

The project is built with a responsibility-driven design.

*Why ?* : It is an Object Oriented concept, I use Abstraction (I think everyone does, no matter the paradigm...) and Encapsulation among the 4 OOP pillars. I think it's a good design for projects when the data is very dynamic.

*How ?* : Here are the main objects and their responsibiities :
- **App** : Responsible of interfacing with the OS. Handles the loop, events, window.

Inside App we got :
- **Model** : Responsible of objects management. It can create, destroy, load or drop them.
- **GraphicsEngine** : Responsible of translating objects to meshes, then rendering them on screen.
- **(For later ?) PhysicsEngine** : Will be responsible of translating objects to bodies, then stepping the physics simulation.

## Graphics engine

Because GraphicsEngine was getting too complex. It was split into subresponsibilities :
- **Presenter** : Responsible of managing the swapchain (quite light).
- **Renderer** : Responsible of rendering meshes (quite heavy).
- **Mesher** : Responsible of creating meshes from objects (light too).

Graphics engine distribute work and handle synchronization.

# Custom vulkan devices

`ash::Device` are wrapped into my own Device objects. It allows to :
- Keep accessible physical device informations (queue family indices, surface capabilities, etc.).
- Provide vulkan extension fns (like the ones in `ash::khr::swapchain::Device`).
- Provide memory-management fns with VMA allocator.
- Provide boilerplate-free fns (creating syncs for example).

The design is currently setup for each "engines" to have its own device.
It's still possible for multiple devices to share the same physical device, I will see later if this design is optimized.