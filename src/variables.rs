#[derive(Debug, Clone, Default)]
pub struct Variables {
    variables: Vec<Variable>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarKey(pub(crate) u64);

impl VarKey {
    #[inline]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum VarKind {
    Constant,
    Variable,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Variable {
    initialized: bool,
    value: f32,
    pub kind: VarKind,
}

impl Variables {
    pub fn new(&mut self, variable: Variable) -> VarKey {
        let len = self.variables.len();
        self.variables.push(variable);
        VarKey(len as u64)
    }

    pub fn prepare(&mut self) {
        for var in &mut self.variables {
            var.prepare();
        }
    }

    pub fn get(&self, key: VarKey) -> Option<f32> {
        if let Some(v) = self.variables.get(key.raw() as usize) {
            v.get()
        } else {
            None
        }
    }

    pub fn set(&mut self, key: VarKey, value: f32) -> Result<f32, VarError> {
        let v = if let Some(v) = self.variables.get_mut(key.raw() as usize) {
            v
        } else {
            return Err(VarError::NotFound);
        };

        v.set(value)
    }

    pub fn set_const(&mut self, key: VarKey, value: f32) -> Result<f32, VarError> {
        let v = if let Some(v) = self.variables.get_mut(key.raw() as usize) {
            v
        } else {
            return Err(VarError::NotFound);
        };

        v.set_const(value)
    }
}

impl Variable {
    pub fn new_var() -> Self {
        Self {
            initialized: false,
            value: 0.0,
            kind: VarKind::Variable,
        }
    }

    pub fn new_const(value: f32) -> Self {
        Self {
            initialized: true,
            value,
            kind: VarKind::Constant,
        }
    }

    pub fn prepare(&mut self) {
        match self.kind {
            VarKind::Constant => (),
            VarKind::Variable => self.initialized = false,
        }
    }

    pub fn get(&self) -> Option<f32> {
        self.initialized.then(|| self.value)
    }

    fn set(&mut self, v: f32) -> Result<f32, VarError> {
        match self.kind {
            VarKind::Constant => return Err(VarError::ConstAssign),
            VarKind::Variable => self.value = v,
        }
        self.initialized = true;
        Ok(self.value)
    }

    pub fn set_const(&mut self, v: f32) -> Result<f32, VarError> {
        match self.kind {
            VarKind::Constant => self.value = v,
            VarKind::Variable => return Err(VarError::ConstAssignOnVariable),
        }
        Ok(self.value)
    }
}

#[derive(Debug)]
pub enum VarError {
    NotFound,
    ConstAssign,
    ConstAssignOnVariable,
}
