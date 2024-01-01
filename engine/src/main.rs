mod game;
mod graphics;
mod wren;

use std::sync::Arc;
use std::time;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::graphics::{vulkan::VulkanGraphicsInterface, GraphicsInterface, Renderable, Vertex};

#[derive(Clone)]
struct StaticVertex {
    point: cgmath::Point3<f32>,
    tex_coords: cgmath::Point2<f32>,
}

impl Vertex for StaticVertex {
    fn get_point(&self) -> cgmath::Point3<f32> {
        self.point
    }

    fn get_tex_coords(&self) -> cgmath::Point2<f32> {
        self.tex_coords
    }
}

struct Triangle {
    vertices: [StaticVertex; 3],
}

impl Renderable<StaticVertex> for Triangle {
    fn get_vertices(&self) -> Vec<StaticVertex> {
        self.vertices.to_vec()
    }

    fn get_indices(&self) -> Vec<u32> {
        vec![0, 1, 2]
    }
}

fn main_graphics() {
    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut graphics_interface = VulkanGraphicsInterface::new(
        &event_loop,
        window.clone(),
        image::open("texture.png").unwrap(),
    );

    graphics_interface.add_renderable(Triangle {
        vertices: [
            StaticVertex {
                point: cgmath::point3(0.0, 1.0, 2.0),
                tex_coords: cgmath::point2(0.5, 0.0),
            },
            StaticVertex {
                point: cgmath::point3(-1.0, -1.0, 2.0),
                tex_coords: cgmath::point2(0.0, 1.0),
            },
            StaticVertex {
                point: cgmath::point3(1.0, -1.0, 2.0),
                tex_coords: cgmath::point2(1.0, 1.0),
            },
        ],
    });

    let mut last_render = time::Instant::now();
    let mut rotation = 0f32;

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
            rotation += 10f32 * last_render.elapsed().as_secs_f32();
            last_render = time::Instant::now();
            graphics_interface.render(graphics::Camera {
                theta_x: 0.0,
                theta_y: rotation.to_radians(),
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

fn main() {
    let write_fn = |_vm: &wren::VM, text: &str| {
        print!("{}", text);
    };

    let error_fn =
        |_vm: &wren::VM, error_type: wren::ErrorType, module: &str, line: i32, msg: &str| {
            match error_type {
                wren::ErrorType::WrenErrorCompile => {
                    println!("[{module} line {line}] [Error] {msg}");
                }
                wren::ErrorType::WrenErrorStackTrace => {
                    println!("[{module} line {line}] in {msg}");
                }
                wren::ErrorType::WrenErrorRuntime => {
                    println!("[Runtime Error] {msg}");
                }
            }
        };

    let config = wren::Configuration {
        write_fn,
        error_fn,
        ..Default::default()
    };

    let vm = wren::VM::new(&config);

    let module = "main";
    let script = "System.print(\"I am running in a VM!\")";

    let result = vm.interpret(Some(module.as_bytes()), script.as_bytes());

    match result {
        Ok(()) => {
            println!("Success!");
        }
        Err(e) => match e {
            wren::InterpretError::CompileError => {
                println!("Compile Error!");
            }
            wren::InterpretError::RuntimeError => {
                println!("Runtime Error!");
            }
        },
    };

    drop(vm);
}
