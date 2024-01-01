mod compiler;
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

pub struct Configuration {}

pub struct VM {}
