use super::{value, value::Value, Configuration, InterpretError};

impl super::VM {
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

    pub fn compile_source(
        &self,
        module: Option<&[u8]>,
        source: &[u8],
        is_expression: bool,
        print_errors: bool,
    ) -> Option<ObjClosure> {
        let name_value = None;
        if module.is_none() {
            name_value = Some(self.new_string(module));
            self.push_root(value::as_obj(name_value))
        }

        let closure = self.compile_in_module(name_value, source, is_expression, print_errors);

        if module.is_none() {
            self.pop_root();
        }

        closure
    }

    pub fn compile_in_module(
        &self,
        name: Option<Value>,
        source: &[u8],
        is_expression: bool,
        print_errors: bool,
    ) -> Option<ObjClosure> {
        let module = self.get_module(name);
        if module.is_none() {
            module = self.new_module(value::as_string(name));

            self.push_root(module);

            self.map_set(self.modules, name, value::obj_val(module));

            self.pop_root();

            let core_module = self.get_module(None);
            for i in 0..core_module.variables.count {
                self.define_variable(
                    module,
                    core_module.variable_names.data[i].value,
                    core_module.variable_names.data[i].length,
                    core_module.variables.data[i],
                    None,
                );
            }
        }

        match self.compile(module, source, is_expression, print_errors) {
            None => None,
            Some(fun) => {
                self.push_root(fun); // TODO fun -> (Obj*)fn
                let closure = self.new_closure(fun);
                self.pop_root();

                closure
            }
        }
    }

    pub fn push_root(&self, obj: &Obj) {
        todo!() // wrenPushRoot
    }

    pub fn pop_root(&self) {
        todo!() // wrepPopRoot
    }
}
