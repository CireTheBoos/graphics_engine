# Introduction

Graphics engine written in Rust using `ash`.

*About Ash* It is a popular crate that provides raw bindings for Vulkan + some quality of life, transparent features. Transparent mean here that there're not black boxes that hide vulkan complexity.
The rawness of it allows me to use the vulkan documentation and C++ vulkan tutorials with little to no problems.

*What is a graphics engine ?* It's a program that computes meshes + camera into a 2D image an render it on screen in real-time. It's a lot about optimization.
It also can simulate light or whatever you want that has an impact on the final image.

# Project structure

The project is built with a responsibility-driven design.

*Why ?* It is an Object Oriented concept, I use Abstraction (I think everyone does, no matter the paradigm...) and Encapsulation among the 4 OOP pillars. I think it's a good design for projects when the data is very dynamic.

*How ?* Here are the main objects and their responsibiities :
- **App** is responsible of interfacing with the OS. Handles loop, events, window.
Inside App we got :
- **Game** is responsible of managing game objects. Handles loading/dropping of them.
- **GraphicsEngine** is responsible of translating game objects to graphics objects, then rendering its graphics model to screen.
- **PhysicsEngine** (TODO) will be responsible of translating game objects to physics objects, then stepping the physics simulation.



