use cgmath::{Matrix3, Matrix4, Rad};
use rayon;
use std::sync::{Arc, Mutex};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned,
        Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex as VertexTrait, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::EntryPoint,
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, future::FenceSignalFuture, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::{event_loop::EventLoop, window::Window};

use crate::graphics::GraphicsInterface;

#[derive(BufferContents, VertexTrait)]
#[repr(C)]
struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
}

struct VulkanRenderable {
    vertex_buffer: Subbuffer<[Vertex]>,
    index_buffer: Subbuffer<[u32]>,
}

pub struct VulkanGraphicsInterface {
    window: Arc<Window>,

    device: Arc<Device>,

    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    swapchain: Arc<Swapchain>,
    uniform_buffer: SubbufferAllocator,

    // Allocators
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    command_buffer_allocator: StandardCommandBufferAllocator,

    // Shaders
    vs: EntryPoint,
    fs: EntryPoint,

    // Render state
    recreate_swapchain: bool,

    // Render pool
    renderable_processing_pool: rayon::ThreadPool,
    renderables: Mutex<Vec<Option<VulkanRenderable>>>,
}

impl GraphicsInterface for VulkanGraphicsInterface {
    fn new(event_loop: &EventLoop<()>, window: Arc<Window>) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(&event_loop);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable device found");

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let (mut swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let image_format = device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0;

            Swapchain::new(
                device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let uniform_buffer = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                depth_stencil: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil}
            }
        )
        .unwrap();

        let vs = vertex_shader::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fragment_shader::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let (mut pipeline, mut framebuffers) = create_pipeline_and_framebuffers(
            memory_allocator.clone(),
            vs.clone(),
            fs.clone(),
            &images,
            render_pass.clone(),
        );
        let mut recreate_swapchain = false;

        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(device.clone(), Default::default());
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        // TODO make thread count configurable
        let renderable_processing_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();

        let renderables = Mutex::new(vec![]);

        Self {
            window,
            device,
            framebuffers,
            pipeline,
            queue,
            render_pass,
            swapchain,
            uniform_buffer,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            vs,
            fs,
            recreate_swapchain: false,
            renderable_processing_pool,
            renderables,
        }
    }

    fn add_renderable(
        &self,
        renderable: super::Renderable,
    ) -> Result<usize, super::AddRenderableError> {
        let index = {
            let renderables = self.renderables.get_mut().unwrap();

            match renderables.iter().position(|x| x.is_none()) {
                Some(idx) => idx,
                None => {
                    renderables.push(None);
                    renderables.len() - 1
                }
            }
        };

        self.renderable_processing_pool.spawn(move || {
            let vulkan_renderable = VulkanRenderable::from(renderable);

            {
                let renderables = self.renderables.get_mut().unwrap();
                renderables[index] = Some(vulkan_renderable);
            };
        });

        Ok(index)
    }

    fn rm_renderable(&self, id: usize) -> Result<(), super::RemoveRenderableError> {
        todo!()
    }

    fn render(&self, camera: super::Camera) -> Result<super::RenderSuccess, super::RenderError> {
        let image_extent: [u32; 2] = self.window.inner_size().into();

        if image_extent.contains(&0) {
            return Ok(super::RenderSuccess::RenderImpossible);
        }

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent,
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = new_swapchain;
            let (new_pipeline, new_framebuffers) = create_pipeline_and_framebuffers(
                self.memory_allocator.clone(),
                self.vs.clone(),
                self.fs.clone(),
                &new_images,
                self.render_pass.clone(),
            );
            self.pipeline = new_pipeline;
            self.framebuffers = new_framebuffers;
            self.recreate_swapchain = false;
        }

        let uniform_buffer_subbuffer = {
            let rotation_x = Matrix3::from_angle_x(Rad(camera.theta_x));
            let rotation_y = Matrix3::from_angle_y(Rad(camera.theta_y));
            let rotation = rotation_x * rotation_y;

            let aspect =
                self.swapchain.image_extent()[0] as f32 / self.swapchain.image_extent()[1] as f32;
            let proj = cgmath::perspective(
                Rad(camera.fov),
                aspect,
                camera.near_cutoff,
                camera.far_cutoff,
            );
            let view = Matrix4::look_at_rh(camera.eye, camera.center, camera.up);
            let scale = Matrix4::from_scale(camera.scale);

            let uniform_data = vertex_shader::Data {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: proj.into(),
            };

            let subbuffer = self.uniform_buffer.allocate_sized().unwrap();
            *subbuffer.write().unwrap() = uniform_data;

            subbuffer
        };

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
            [],
        )
        .unwrap();

        let (image_index, suboptimal, acquire_feature) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(super::RenderSuccess::RenderImpossible);
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let mut command_buffer_builder = builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into()), Some(1f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                Default::default(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set,
            )
            .unwrap();

        {
            let renderables = self.renderables.lock().unwrap();

            for renderable in renderables.iter() {
                match renderable {
                    Some(renderable) => {
                        command_buffer_builder = command_buffer_builder
                            .bind_vertex_buffers(0, renderable.vertex_buffer.clone())
                            .unwrap()
                            .bind_index_buffer(renderable.index_buffer.clone())
                            .unwrap()
                            .draw_indexed(renderable.index_buffer.len() as u32, 1, 0, 0, 0)
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }

        command_buffer_builder = command_buffer_builder
            .end_render_pass(Default::default())
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let execution = sync::now(self.device.clone())
            .join(acquire_feature)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => {
                future.wait(None).unwrap();
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                println!("failed to flush future: {e}");
            }
        }

        Ok(super::RenderSuccess::Rendered)
    }

    fn on_resized(
        &self,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<(), super::ResizeError> {
        self.recreate_swapchain = true;
        Ok(())
    }
}

fn create_pipeline_and_framebuffers(
    memory_allocator: Arc<StandardMemoryAllocator>,
    vs: EntryPoint,
    fs: EntryPoint,
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let device = memory_allocator.device().clone();
    let extent = images[0].extent();

    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let pipeline = {
        let vertex_input_state = Vertex::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
        let subpass = Subpass::from(render_pass, 0).unwrap();

        GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [Viewport {
                        offset: [0.0, 0.0],
                        extent: [extent[0] as f32, extent[1] as f32],
                        depth_range: 0.0..=1.0,
                    }]
                    .into_iter()
                    .collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    (pipeline, framebuffers)
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                mat4 worldview = uniforms.view * uniforms.world;
                gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
            }
        ",
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "
    }
}
