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

## Graphics engine

Because GraphicsEngine was getting too complex. It was split into subresponsibilities :
- **Presenter** : Responsible of managing the swapchain.
- **Mesher** : Responsible of creating meshes from the model.
- **Renderer** : Responsible of rendering meshes (quite heavy compared to the other 2).

GraphicsEngine object distribute work and handle synchronization.

# Custom vulkan devices

This is one of my favorite design as I find having an app-specific mighty device super practical.
`ash::Device` are wrapped into Device objects. It allows to :
- Keep accessible physical device informations (queue family indices, surface capabilities, etc.).
- Provide vulkan extension fns (like the ones in `ash::khr::swapchain::Device`).
- Provide custom memory-management fns with VMA allocator.
- Provide boilerplate-free fns (creating syncs for example).

Check the one in src/app/graphics_engine/device.rs