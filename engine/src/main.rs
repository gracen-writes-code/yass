mod graphics;

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::graphics::{vulkan::VulkanGraphicsInterface, GraphicsInterface};

const FOV: f32 = 70.0f32.to_radians();
const NEAR_CUTOFF: f32 = 0.01;
const FAR_CUTOFF: f32 = 100.0;

fn main() {
    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let graphics_interface = VulkanGraphicsInterface::new(&event_loop, window.clone());

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            event: window_event,
        } if window_id == window.id() => match window_event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(new_size) => {
                graphics_interface.on_resized(new_size);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            graphics_interface.render(camera);
        }
        _ => {}
    });
}
