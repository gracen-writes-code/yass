mod graphics;

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::graphics::{vulkan::VulkanGraphicsInterface, GraphicsInterface, Renderable};

struct Triangle {
    vertices: [cgmath::Point3<f32>; 3],
}

impl Renderable for Triangle {
    fn get_vertices(&self) -> Vec<cgmath::Point3<f32>> {
        self.vertices.to_vec()
    }

    fn get_indices(&self) -> Vec<u32> {
        vec![0, 1, 2]
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut graphics_interface = VulkanGraphicsInterface::new(&event_loop, window.clone());

    graphics_interface.add_renderable(Triangle {
        vertices: [
            cgmath::point3(1.0, 1.0, 2.0),
            cgmath::point3(-1.0, -1.0, 2.0),
            cgmath::point3(1.0, -1.0, 2.0),
        ],
    });

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
            graphics_interface.render(graphics::Camera {
                theta_x: 0.0,
                theta_y: 30.0f32.to_radians(),
                fov: 70.0f32.to_radians(),
                near_cutoff: 0.01,
                far_cutoff: 100.0,
                eye: cgmath::point3(0.0, 0.0, 0.0),
                center: cgmath::point3(0.0, 0.0, 1.0),
                up: cgmath::vec3(0.0, 1.0, 0.0),
                scale: 1.0,
            });
        }
        _ => {}
    });
}
