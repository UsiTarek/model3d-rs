use super::*;

pub type Weight = m3dc::m3dw_t;

#[repr(transparent)]
pub struct Bone(m3dc::m3db_t);

impl Bone {
    pub fn parent(&self) -> u32 {
        self.0.parent
    }

    pub fn name(&self) -> &str {
        unsafe {
            let name = self.0.name;
            if name.is_null() {
                ""
            } else {
                cptr_to_str(self.0.name)
            }
        }
    }

    pub fn position(&self) -> u32 {
        self.0.pos
    }

    pub fn orientation(&self) -> u32 {
        self.0.ori
    }

    pub fn weights(&self) -> &[Weight] {
        unsafe { cptr_to_slice(self.0.weight, self.0.numweight as _) }
    }

    pub fn mat4(&self) -> &[f32; 16usize] {
        &self.0.mat4
    }
}
