use super::*;

pub type MaterialProp = m3dc::m3dp_t;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Material(m3dc::m3dm_t);

impl Material {
    pub fn name(&self) -> &str {
        unsafe { cptr_to_str(self.0.name) }
    }

    pub fn props(&self) -> &[MaterialProp] {
        unsafe { cptr_to_slice(self.0.prop, self.0.numprop as _) }
    }
}
