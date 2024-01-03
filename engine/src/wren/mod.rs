mod common;
mod compiler;
mod utils;
mod value;
mod vm;

pub enum ErrorType {
    WrenErrorCompile,
    WrenErrorRuntime,
    WrenErrorStackTrace,
}

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub enum WrenType {
    Bool,
    Num,
    Foreign,
    List,
    Map,
    Null,
    String,

    Unknown,
}

type InterpretResult = Option<(), InterpretError>;

pub struct Configuration {
    pub reallocate_fn: Option<ReallocateFn>, // TODO this should not be needed, investigate when i get around to GC shit
    pub resolve_module_fn: Option<ResolveModuleFn>,
    pub load_module_fn: LoadModuleFn,
    pub bind_foreign_method_fn: BindForeignMethodFn,
    pub bind_foreign_class_fn: BindForeignClassFn,
    pub write_fn: Option<WriteFn>,
    pub error_fn: Option<ErrorFn>,

    pub initial_heap_size: Option<usize>,
    pub min_heap_size: Option<usize>,
    pub heap_growth_percent: Option<i32>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            reallocate_fn: None,
            resolve_module_fn: None,
            load_module_fn: todo!(),
            bind_foreign_method_fn: todo!(),
            bind_foreign_class_fn: todo!(),
            write_fn: None,
            error_fn: None,
            initial_heap_size: None,
            min_heap_size: None,
            heap_growth_percent: None,
        }
    }
}

pub struct VM {
    config: Configuration,
}

impl VM {
    pub fn new(config: &Configuration) -> Self {
        todo!() // wrenNewVM
    }

    pub fn collect_garbage(&self) {
        todo!(); // wrenCollectGarbage
    }

    pub fn interpret(&self, module: Option<&[u8]>, source: &[u8]) -> InterpretResult {
        let closure = self.compile_source(module, source, false, true);
        if closure.is_none() {
            return Err(InterpretError::CompileError);
        }

        self.push_root(closure);
        let fiber = self.new_fiber(closure);
        self.pop_root();
        self.api_stack = None;

        self.run_interpreter(fiber)
    }

    pub fn make_call_handle(&self, signature: &[u8]) -> Handle {
        todo!() // wrenMakeCallHandle
    }

    pub fn call(&self, method: &Handle) -> InterpretResult {
        todo!() // wrenCall
    }

    pub fn release_handle(&self, handle: &Handle) {
        todo!(); // wrenReleaseHandle
    }

    pub fn get_slot_count(&self) -> i32 {
        todo!() // wrenGetSlotCount
    }

    pub fn ensure_slots(&self, num_slots: i32) -> Result<(), EnsureSlotsError> {
        todo!(); // wrenEnsureSlots
    }

    pub fn get_slot_type(&self, slot: i32) -> WrenType {
        todo!() // wrenGetSlotType
    }

    pub fn get_slot_bool(&self, slot: i32) -> Result<bool, GetSlotError> {
        todo!() // wrenGetSlotBool
    }

    pub fn get_slot_bytes(&self, slot: i32) -> Result<(&[u8], usize), GetSlotError> {
        todo!() // wrenGetSlotBytes
    }

    pub fn get_slot_double(&self, slot: i32) -> Result<f64, GetSlotError> {
        todo!() // wrenGetSlotDouble
    }

    // TODO figure out how to implement wrenGetSlotForeign without void*

    // TODO add stubs for everything past wrenGetSlotForeign
}

impl Drop for VM {
    fn drop(&mut self) {
        todo!(); // wrenFreeVM, may be unnecessary
    }
}

pub fn get_version_number() -> i32 {
    todo!() // wrenGetVersionNumber
}
