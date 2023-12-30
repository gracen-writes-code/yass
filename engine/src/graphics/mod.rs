pub mod vulkan;

use std::sync::Arc;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

pub enum RenderSuccess {
    Rendered,

    RenderImpossible,
}
pub enum RenderError {}

pub enum ResizeError {}

pub enum AddRenderableError {}

pub enum RemoveRenderableError {}

pub struct Camera {
    theta_x: f32,
    theta_y: f32,

    fov: f32,
    near_cutoff: f32,
    far_cutoff: f32,

    eye: cgmath::Point3<f32>,
    center: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,

    scale: f32,
}

pub trait GraphicsInterface {
    fn new(event_loop: &EventLoop<()>, window: Arc<Window>) -> Self;

    fn add_renderable(&self, renderable: Renderable) -> Result<usize, AddRenderableError>;
    fn rm_renderable(&self, id: usize) -> Result<(), RemoveRenderableError>;

    fn render(&self, camera: Camera) -> Result<RenderSuccess, RenderError>;

    fn on_resized(&self, new_size: PhysicalSize<u32>) -> Result<(), ResizeError>;
}
