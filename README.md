# Device

My Devices wraps `ash::Device` to :
- Keep accessible physical device informations (queue family indices, surface capabilities, etc.).
- Easily provide extension vulkan fns (They are impl in another struct in Ash, like `ash::khr::swapchain::Device`).
- Provide memory-management fns with VMA allocator.

Each independent "Engine" has its own Device.

# Structure

The project is built with a responsibility-driven design.

*Why ?* It is an Object Oriented concept, I use Abstraction (I think everyone does, no matter the paradigm...) and Encapsulation among the 4 OOP pillars. I think it's a good design for projects when the data is very dynamic.

*How ?* Here are the main objects and their responsibiities :
- **App** is responsible of interfacing with the OS. Handles network, game loop (user inputs), window.
Inside App we got :
- **Model** is responsible of managing in-game objects. Handles loading/dropping of in-game objects.
- **GraphicsEngine** is responsible of rendering Model to screen. Handles image rendering and presentation.
- **PhysicsEngine** is responsible of updating Model. Handles movements, collisions, etc.



