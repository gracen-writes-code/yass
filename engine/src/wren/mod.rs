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

    pub fn interpret(&self, module: Option<&[u8]>, source: &[u8]) -> Result<(), InterpretError> {
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
}
