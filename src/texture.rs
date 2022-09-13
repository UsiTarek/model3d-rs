use super::*;
use num_enum::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum TextureFormat {
    #[num_enum(default)]
    Invalid = 0,
    Grayscale = 1,
    GrayscaleAndAlpha = 2,
    RGB = 3,
    RGBA = 4,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Texture(m3dc::m3dtx_t);

impl Texture {
    pub fn data(&self) -> &[u8] {
        let bytes_len =
            (self.width() as usize) * (self.height() as usize) * (self.format() as usize);
        unsafe { cptr_to_slice(self.0.d, bytes_len as _) }
    }

    pub fn name(&self) -> &str {
        unsafe {
            let name = self.0.name;
            if name.is_null() {
                ""
            } else {
                cptr_to_str(name)
            }
        }
    }

    pub fn width(&self) -> u16 {
        self.0.w
    }

    pub fn height(&self) -> u16 {
        self.0.h
    }

    pub fn format(&self) -> TextureFormat {
        self.0.f.into()
    }
}
