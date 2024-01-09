pub mod vulkan;

use std::sync::{Arc, Mutex};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

pub struct Camera {
    pub theta_x: f32,
    pub theta_y: f32,

    pub fov: f32,
    pub near_cutoff: f32,
    pub far_cutoff: f32,

    pub eye: cgmath::Point3<f32>,
    pub center: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,

    pub scale: f32,
}

pub trait Vertex {
    fn get_point(&self) -> cgmath::Point3<f32>;
    fn get_tex_coords(&self) -> cgmath::Point2<f32>;
}

pub trait Renderable<V: Vertex> {
    fn get_vertices(&self) -> Vec<V>;
    fn get_indices(&self) -> Vec<u32>;
}

pub trait GraphicsInterface {
    fn new(event_loop: EventLoop<()>, window: Window, texture: image::DynamicImage) -> Self;

    fn add_renderable<V: Vertex>(&mut self, renderable: impl Renderable<V> + Send) -> usize;
    fn rm_renderable(&mut self, id: usize);

    fn render(&mut self, camera: Camera);

    fn on_resized(&mut self, new_size: PhysicalSize<u32>);

    fn event_loop()
}

pub struct GraphicsLoop<I: GraphicsInterface> {
    interface: I
}

impl GraphicsLoop {
    fn new(interface: dyn GraphicsInterface) -> Self {
        todo!()
    }
}