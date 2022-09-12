use super::*;

#[repr(transparent)]
pub struct Obj(m3dc::m3d_t);

impl Drop for Obj {
    fn drop(&mut self) {
        unsafe {
            let m3d_ptr = std::mem::transmute::<_, *mut m3dc::m3d_t>(self);
            m3dc::m3d_free(m3d_ptr);
        }
    }
}

impl Obj {
    pub fn load_from_u8<'a>(
        data: &mut Vec<u8>,
        mtllib: Option<&mut Obj>,
    ) -> Result<&'a Obj, Error> {
        let mttlib_c = match mtllib {
            Some(mtl) => mtl as *mut Obj as *mut m3dc::m3d_t,
            None => std::ptr::null_mut(),
        };

        //#TODO: Using libc to read file to buffer and free it, expose proper rust closures.
        let m3d_c = unsafe {
            m3dc::m3d_load(
                data.as_mut_ptr(),
                Some(m3dread_default),
                Some(libc::free),
                mttlib_c,
            )
        };

        if m3d_c.is_null() {
            return Err(Error::ReturnedNull);
        }

        let err = unsafe { Error::from((*m3d_c).errcode) };
        if err as i8 == m3dc::M3D_SUCCESS as i8 {
            Ok(unsafe { &*(m3d_c as *const Obj) })
        } else {
            Err(err)
        }
    }

    pub fn load_from_file<P: AsRef<std::path::Path>>(
        path: P,
        mtllib: Option<&mut Obj>,
    ) -> Result<&Obj, Error> {
        let result = std::fs::read(path);
        match result {
            Err(_) => Err(Error::ReturnedNull),
            Ok(mut data) => Self::load_from_u8(&mut data, mtllib),
        }
    }

    pub fn save(
        &self,
        quality: Option<QuantizeQuality>,
        flags: Option<SaveFlags>,
    ) -> Option<&[u8]> {
        let mut m3d_encoded_len = 0u32;
        let m3d_encoded = unsafe {
            m3d_save(
                // Casting *const Object to *mut m3dc::m3d_t since only model::errorcode is modified in C
                self as *const Obj as *mut m3dc::m3d_t,
                quality.unwrap_or(QuantizeQuality::F32) as _,
                if let Some(flags) = flags {
                    flags.bits
                } else {
                    0
                },
                &mut m3d_encoded_len as _,
            )
        };

        if m3d_encoded.is_null() {
            None
        } else {
            unsafe { Some(cptr_to_slice(m3d_encoded, m3d_encoded_len as _)) }
        }
    }

    pub fn name(&self) -> &str {
        unsafe { cptr_to_str(self.0.name) }
    }

    pub fn flags(&self) -> ObjectFlags {
        ObjectFlags::from_bits(self.0.flags as u8).unwrap()
    }

    pub fn license(&self) -> &str {
        unsafe { cptr_to_str(self.0.license) }
    }

    pub fn author(&self) -> &str {
        unsafe { cptr_to_str(self.0.author) }
    }

    pub fn desc(&self) -> &str {
        unsafe { cptr_to_str(self.0.desc) }
    }

    pub fn color_maps(&self) -> &[u32] {
        unsafe { cptr_to_slice(self.0.cmap, self.0.numcmap as _) }
    }

    pub fn texture_maps(&self) -> &[TextureMapIndex] {
        unsafe { cptr_to_slice(self.0.tmap, self.0.numtmap as _) }
    }

    pub fn textures(&self) -> &[Texture] {
        let slice = unsafe { cptr_to_slice(self.0.texture, self.0.numtexture as _) };
        unsafe { std::mem::transmute::<_, &[Texture]>(slice) }
    }

    pub fn bones(&self) -> &[Bone] {
        let slice = unsafe { cptr_to_slice(self.0.bone, self.0.numbone as _) };
        unsafe { std::mem::transmute::<_, &[Bone]>(slice) }
    }

    pub fn vertices(&self) -> &[Vertex] {
        unsafe { cptr_to_slice(self.0.vertex, self.0.numvertex as _) }
    }

    pub fn skins(&self) -> &[Skin] {
        unsafe { cptr_to_slice(self.0.skin, self.0.numskin as _) }
    }

    pub fn materials(&self) -> &[Material] {
        let slice = unsafe { cptr_to_slice(self.0.material, self.0.nummaterial as _) };
        unsafe { std::mem::transmute::<_, &[Material]>(slice) }
    }

    pub fn faces(&self) -> &[Face] {
        unsafe { cptr_to_slice(self.0.face, self.0.numface as _) }
    }

    pub fn actions(&self) -> &[Action] {
        let slice = unsafe { cptr_to_slice(self.0.action, self.0.numaction as _) };
        unsafe { std::mem::transmute::<_, &[Action]>(slice) }
    }

    pub fn inlined_textures(&self) -> &[InlinedTexture] {
        let slice = unsafe { cptr_to_slice(self.0.inlined, self.0.numinlined as _) };
        unsafe { std::mem::transmute::<_, &[InlinedTexture]>(slice) }
    }

    pub fn inlined(&self) -> &InlinedTexture {
        unsafe { std::mem::transmute::<_, &InlinedTexture>(&self.0.preview) }
    }
}
