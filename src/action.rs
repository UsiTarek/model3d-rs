use super::*;

pub struct Frame {
    c: m3dc::m3dfr_t,
}

impl Frame {
    pub fn msec(&self) -> u32 {
        self.c.msec
    }

    pub fn transforms(&self) -> &[Transform] {
        unsafe { cptr_to_slice(self.c.transform, self.c.numtransform as _) }
    }
}

#[repr(C)]
pub struct Action {
    c: m3dc::m3da_t,
}

impl Action {
    pub fn name(&self) -> &str {
        unsafe { cptr_to_str(self.c.name) }
    }

    pub fn duraction_msec(&self) -> u32 {
        self.c.durationmsec
    }

    pub fn frames(&self) -> &[Frame] {
        let slice = unsafe { cptr_to_slice(self.c.frame, self.c.numframe as _) };
        unsafe { std::mem::transmute::<_, &[Frame]>(slice) }
    }
}
