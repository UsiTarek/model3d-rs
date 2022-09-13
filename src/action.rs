use super::*;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Frame(m3dc::m3dfr_t);

impl Frame {
    pub fn msec(&self) -> u32 {
        self.0.msec
    }

    pub fn transforms(&self) -> &[Transform] {
        unsafe { cptr_to_slice(self.0.transform, self.0.numtransform as _) }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Action(m3dc::m3da_t);

impl Action {
    pub fn name(&self) -> &str {
        unsafe { cptr_to_str(self.0.name) }
    }

    pub fn duraction_msec(&self) -> u32 {
        self.0.durationmsec
    }

    pub fn frames(&self) -> &[Frame] {
        let slice = unsafe { cptr_to_slice(self.0.frame, self.0.numframe as _) };
        unsafe { std::mem::transmute::<_, &[Frame]>(slice) }
    }
}
