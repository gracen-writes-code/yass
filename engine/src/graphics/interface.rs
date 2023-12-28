use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::{Validated, VulkanError, VulkanLibrary};

pub enum LoadingError {
    NoSuitableDevice,
    NoSuitableQueueFamily,

    VulkanError(VulkanError),
    VulkanLoadingError(vulkano::LoadingError),
    VulkanValidationError(Box<vulkano::ValidationError>),
}

impl From<vulkano::LoadingError> for LoadingError {
    fn from(value: vulkano::LoadingError) -> Self {
        LoadingError::VulkanLoadingError(value)
    }
}

impl From<vulkano::VulkanError> for LoadingError {
    fn from(value: vulkano::VulkanError) -> Self {
        LoadingError::VulkanError(value)
    }
}

impl From<Validated<VulkanError>> for LoadingError {
    fn from(value: Validated<VulkanError>) -> Self {
        match value {
            Validated::Error(e) => return LoadingError::VulkanError(e),
            Validated::ValidationError(e) => return LoadingError::VulkanValidationError(e),
        }
    }
}

pub struct GraphicsInterface {}

impl GraphicsInterface {
    pub fn load() -> Result<GraphicsInterface, LoadingError> {
        let library = VulkanLibrary::new()?;
        let instance = Instance::new(library, InstanceCreateInfo::default())?;

        let physical_device = instance
            .enumerate_physical_devices()?
            .next() // TODO add a method to select device
            .ok_or(LoadingError::NoSuitableDevice)?;

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .ok_or(LoadingError::NoSuitableQueueFamily)? as u32;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        todo!();
    }
}
