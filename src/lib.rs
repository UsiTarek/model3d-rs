#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod bindings;
use bindings as m3dc;

pub type Vertex = m3dc::m3dv_t;
pub type Skin = m3dc::m3ds_t;
pub type Face = m3dc::m3df_t;
pub type Transform = m3dc::m3dtr_t;
pub type TextureMapIndex = m3dc::m3dti_t;

pub mod action;
pub mod bone;
pub mod inlined_texture;
pub mod material;
pub mod texture;

pub use action::*;
pub use bitflags::bitflags;
pub use bone::*;
pub use inlined_texture::*;
pub use material::*;
pub use object::*;
pub use texture::*;

pub mod object;

use std::ffi::CStr;
use std::ptr;

use self::bindings::m3d_save;

bitflags! {
    pub struct SaveFlags : i32 {
        const NO_CMAP = m3dc::M3D_EXP_NOCMAP as _;
        const NO_MATERIAL = m3dc::M3D_EXP_NOMATERIAL as _;
        const NO_FACE = m3dc::M3D_EXP_NOFACE as _;
        const NO_NORMAL = m3dc::M3D_EXP_NONORMAL as _;
        const NO_TEXCOORD = m3dc::M3D_EXP_NOTXTCRD as _;
        const FLIP_TEXCOORD = m3dc::M3D_EXP_FLIPTXTCRD as _;
        const NO_RECALC = m3dc::M3D_EXP_NORECALC as _;
        const IDOSUCK   = m3dc::M3D_EXP_IDOSUCK as _;
        const NO_BONE =  m3dc::M3D_EXP_NOBONE as _;
        const NO_ACTION = m3dc::M3D_EXP_NOACTION as _;
        const INLINE =  m3dc::M3D_EXP_INLINE as _;
        const EXTRA = m3dc::M3D_EXP_EXTRA as _;
        const NO_ZLIB = m3dc::M3D_EXP_NOZLIB as _;
        const ASCII = m3dc::M3D_EXP_ASCII as _;
        const NO_VRT_MAX = m3dc::M3D_EXP_NOVRTMAX as _;
    }
}

bitflags! {
    pub struct ObjectFlags : u8 {
        const FREE_RAW = 0b0001;
        const FREE_STR = 0b0010;
        const MTLLIB = 0b0100;
        const GEN_NORM = 0b1000;
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, num_enum::FromPrimitive)]
pub enum QuantizeQuality {
    #[num_enum(default)]
    Int8 = m3dc::M3D_EXP_INT8 as _,
    Int16 = m3dc::M3D_EXP_INT16 as _,
    F32 = m3dc::M3D_EXP_FLOAT as _,
    F64 = m3dc::M3D_EXP_DOUBLE as _,
}

#[repr(i8)]
#[derive(Debug, Copy, Clone, PartialEq, num_enum::FromPrimitive)]
pub enum Error {
    Alloc = m3dc::M3D_ERR_ALLOC as _,
    BadFile = m3dc::M3D_ERR_BADFILE as _,
    Unimplemented = m3dc::M3D_ERR_UNIMPL as _,
    UnkownProperty = m3dc::M3D_ERR_UNKPROP as _,
    UnknownMesh = m3dc::M3D_ERR_UNKMESH as _,
    UnknownImg = m3dc::M3D_ERR_UNKIMG as _,
    UnknownFrame = m3dc::M3D_ERR_UNKFRAME as _,
    UnknownCmd = m3dc::M3D_ERR_UNKCMD as _,
    UnknownkVoxex = m3dc::M3D_ERR_UNKVOX as _,
    Truncating = m3dc::M3D_ERR_TRUNC as _,
    ColorMap = m3dc::M3D_ERR_CMAP as _,
    TextureMap = m3dc::M3D_ERR_TMAP as _,
    Vertices = m3dc::M3D_ERR_VRTS as _,
    Bone = m3dc::M3D_ERR_BONE as _,
    Material = m3dc::M3D_ERR_MTRL as _,
    Shape = m3dc::M3D_ERR_SHPE as _,
    VoxelType = m3dc::M3D_ERR_VOXT as _,

    /** Additional Errors for the Rust wrapper */
    ReturnedNull = i8::MIN,
    FileNotFound = i8::MAX,

    #[num_enum(default)]
    UnknownError = 0,
}

unsafe fn cptr_to_str<'a>(cstr_ptr: *const i8) -> &'a str {
    assert!(!cstr_ptr.is_null());
    let result = CStr::from_ptr(cstr_ptr).to_str();
    if let Some(str) = result.ok() {
        str
    } else {
        ""
    }
}

unsafe fn cptr_to_slice<'a, T>(cptr: *const T, len: usize) -> &'a [T] {
    assert!(len < std::isize::MAX as _);
    if cptr != ptr::null_mut() || len == 0 {
        std::slice::from_raw_parts(cptr, len as _)
    } else {
        &[]
    }
}

unsafe extern "C" fn m3dread_default(
    filename: *mut libc::c_char,
    size: *mut libc::c_uint,
) -> *mut libc::c_uchar {
    let mut ret: *mut u8 = std::ptr::null_mut();

    let mode = std::ffi::CString::new("rb").unwrap();
    let file = libc::fopen(filename, mode.as_ptr());

    if !file.is_null() {
        libc::fseek(file, 0, libc::SEEK_END);
        *size = libc::ftell(file) as _;
        libc::fseek(file, 0, libc::SEEK_SET);
        ret = libc::malloc((*size + 1) as _) as _;
        if ret != std::ptr::null_mut() {
            libc::fread(ret as _, *size as _, 1, file);
            *(ret.offset(*size as _)) = 0;
        } else {
            *size = 0;
        }
        libc::fclose(file);
    }

    ret
}
