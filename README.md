Graphics (+ maybe Physics) engine written in Rust using `ash`.

## About ash

It is a popular crate that provides raw bindings for Vulkan + some quality of life, transparent features.
"transparent" means here that they're not black boxes that hide vulkan complexity.
The rawness of it allows me to use the vulkan documentation and C++ vulkan tutorials with little to no problems.

The problems (annoying but not hard to solve) came from :
- How ash differs from the API. Solution : Reading ash's README is enough most of the time.
- Finding Rust's equivalents of specific C fns or other libraries (to use Vulkan Memory Allocator or GLSL Math).

## Goals

*What is a graphics engine ?* : It's a program that computes meshes + camera into a 2D image and render it on screen in real-time.
It also can simulate light or whatever you want that has an impact on the final image.

*What is a physics engine ?* : It's a program that computes body movements and behaviours.

**Goal** : Make a 3D physics simulation + graphics rendering. This project was made with the idea of a 3D game engine.

More information about the project technical organization in DOC.md
