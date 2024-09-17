mod instance;
mod renderer;

use instance::Instance;
use renderer::Renderer;

use std::{marker::PhantomPinned, pin::pin};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
    window::WindowId,
};

// Holds application's technical details
// Warning : "renderer" should drop before "instance", hence this field order
struct App {
    renderer: Option<Renderer>,
    instance: Instance,
    _pin: PhantomPinned,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> App {
        let raw_display_handle: RawDisplayHandle = event_loop
            .owned_display_handle()
            .display_handle()
            .expect("Failed to get display handle.")
            .into();
        App {
            _pin: PhantomPinned,
            instance: Instance::new(raw_display_handle),
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.renderer = Some(Renderer::new(event_loop, &self.instance));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.renderer.as_ref().unwrap().window.request_redraw();
            }
            _event => {
                // println!("{:?}",_event);
            }
        }
    }
}

fn main() {
    // STEP 1. : Setup the event_loop to receive events from the OS and window
    let event_loop = EventLoop::new().expect("Failed to create event loop.");
    event_loop.set_control_flow(ControlFlow::Poll);

    // STEP 2. : Create App and run it with the event_loop
    // safe bc run_app() don't move "app" with the &mut given, so pinning hold
    let app = App::new(&event_loop);
    let app = pin!(app);
    event_loop
        .run_app(unsafe { app.get_unchecked_mut() })
        .expect("Failed to run the app.");
}
