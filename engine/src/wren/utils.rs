use super::VM;

pub(super) struct Buffer<'a, T> {
    pub(super) data: Option<&'a [T]>,
    pub(super) count: i32,
    pub(super) capacity: i32,
}

impl<T> Buffer<'_, T> {
    pub(super) fn new() -> Self {
        Self {
            data: None,
            count: 0,
            capacity: 0,
        }
    }

    pub(super) fn write(&self, vm: &VM, data: T) {
        vm.buffer_write(self, data)
    }

    pub(super) fn clear(&self, vm: &VM) {
        vm.buffer_clear(self)
    }
}

impl super::VM {
    pub(super) fn buffer_write<T>(&self, buffer: &Buffer<T>, data: T) {
        todo!()
    }

    pub(super) fn buffer_clear<T>(&self, buffer: &Buffer<T>) {
        todo!()
    }
}
