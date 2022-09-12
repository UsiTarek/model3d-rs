use super::*;

#[repr(transparent)]
pub struct InlinedTexture(m3dc::m3di_t);

impl InlinedTexture {
    pub fn name(&self) -> &str {
        unsafe { cptr_to_str(self.0.name) }
    }

    pub fn data(&self) -> &[u8] {
        unsafe { cptr_to_slice(self.0.data, self.0.length as _) }
    }
}
