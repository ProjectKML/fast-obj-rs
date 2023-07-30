extern crate core;

use std::{
    error::Error,
    ffi::{CString, NulError},
    fmt::Display,
    os::raw::{c_char, c_ulong},
    path::Path,
    ptr, slice, str
};

use enum_display_derive::Display;
use libc::{c_void, strlen};

use crate::{ffi::size_t, ObjLoadError::InvalidPath};

mod ffi;

#[derive(Debug, Display)]
pub enum ObjLoadError {
    InvalidPath,
    ParsingFailed
}

impl Error for ObjLoadError {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Index {
    pub p: u32,
    pub t: u32,
    pub n: u32
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Group {
    name: *mut c_char,
    pub face_count: u32,
    pub face_offset: u32,
    pub index_offset: u32
}

#[inline]
unsafe fn str_from_cstr<'a>(cstr: *mut c_char) -> &'a str {
    std::str::from_utf8_unchecked(slice::from_raw_parts(cstr as *const u8, strlen(cstr)))
}

impl Group {
    pub fn name(&self) -> &str {
        unsafe { str_from_cstr(self.name) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Texture {
    name: *mut c_char,
    path: *mut c_char
}

impl Texture {
    pub fn name(&self) -> &str {
        unsafe { str_from_cstr(self.name) }
    }

    pub fn path(&self) -> &str {
        unsafe { str_from_cstr(self.path) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Material {
    name: *mut c_char,
    pub ka: [f32; 3],
    pub kd: [f32; 3],
    pub ks: [f32; 3],
    pub ke: [f32; 3],
    pub kt: [f32; 3],
    pub ns: f32,
    pub ni: f32,
    pub tf: [f32; 3],
    pub d: f32,
    pub illum: i32,
    pub map_ka: Texture,
    pub map_kd: Texture,
    pub map_ks: Texture,
    pub map_ke: Texture,
    pub map_kt: Texture,
    pub map_ns: Texture,
    pub map_ni: Texture,
    pub map_d: Texture,
    pub map_bump: Texture
}

impl Material {
    pub fn name(&self) -> &str {
        unsafe { str_from_cstr(self.name) }
    }
}

impl From<NulError> for ObjLoadError {
    fn from(_: NulError) -> Self {
        InvalidPath
    }
}

pub type Callbacks = ffi::fastObjCallbacks;

pub struct Mesh {
    mesh: *mut ffi::fastObjMesh
}

impl Mesh {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ObjLoadError> {
        unsafe {
            let path_string = CString::new(path.as_ref().to_str().ok_or(ObjLoadError::InvalidPath)?)?;

            let mesh = ffi::fast_obj_read(path_string.as_ptr());
            if mesh.is_null() {
                Err(ObjLoadError::ParsingFailed)
            } else {
                Ok(Self { mesh })
            }
        }
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self, ObjLoadError> {
        struct UserData<'a> {
            bytes: &'a [u8],
            read_offset: usize
        }

        let mut user_data = UserData { bytes, read_offset: 0 };

        unsafe extern "C" fn file_open(_path: *const c_char, _user_data: *mut c_void) -> *mut c_void {
            _user_data
        }

        unsafe extern "C" fn file_close(_file: *mut c_void, _user_data: *mut c_void) {}

        unsafe extern "C" fn file_read(_file: *mut c_void, dst: *mut c_void, bytes: size_t, user_data: *mut c_void) -> size_t {
            let user_data = &mut *{ user_data as *mut UserData };
            libc::memcpy(dst, user_data.bytes.as_ptr().add(user_data.read_offset).cast(), bytes as _);
            user_data.read_offset += bytes as usize;
            bytes as _
        }

        unsafe extern "C" fn file_size(_file: *mut c_void, user_data: *mut c_void) -> c_ulong {
            let user_data = &mut *{ user_data as *mut UserData };
            user_data.bytes.len() as _
        }

        let callbacks = Callbacks {
            file_open: Some(file_open),
            file_close: Some(file_close),
            file_read: Some(file_read),
            file_size: Some(file_size)
        };

        unsafe { Self::new_with_callbacks("", &callbacks, &mut user_data as *mut _ as *mut _) }
    }

    pub unsafe fn new_with_callbacks<P: AsRef<Path>>(path: P, callbacks: &Callbacks, user_data: *mut c_void) -> Result<Self, ObjLoadError> {
        let path_string = CString::new(path.as_ref().to_str().ok_or(ObjLoadError::InvalidPath)?)?;

        let mesh = ffi::fast_obj_read_with_callbacks(path_string.as_ptr(), callbacks as *const Callbacks, user_data);

        if mesh.is_null() {
            Err(ObjLoadError::ParsingFailed)
        } else {
            Ok(Self { mesh })
        }
    }

    pub fn positions(&self) -> &[f32] {
        unsafe { slice::from_raw_parts((*self.mesh).positions, 3 * (*self.mesh).position_count as usize) }
    }

    pub fn texcoords(&self) -> &[f32] {
        unsafe { slice::from_raw_parts((*self.mesh).texcoords, 2 * (*self.mesh).texcoord_count as usize) }
    }

    pub fn normals(&self) -> &[f32] {
        unsafe { slice::from_raw_parts((*self.mesh).normals, 3 * (*self.mesh).normal_count as usize) }
    }

    pub fn face_vertices(&self) -> &[u32] {
        unsafe { slice::from_raw_parts((*self.mesh).face_vertices, (*self.mesh).face_count as usize) }
    }

    pub fn face_materials(&self) -> &[u32] {
        unsafe { slice::from_raw_parts((*self.mesh).face_materials, (*self.mesh).face_count as usize) }
    }

    pub fn indices(&self) -> &[Index] {
        unsafe { slice::from_raw_parts((*self.mesh).indices as *mut Index, (*self.mesh).index_count as usize) }
    }

    pub fn materials(&self) -> &[Material] {
        unsafe { slice::from_raw_parts((*self.mesh).materials as *mut Material, (*self.mesh).material_count as usize) }
    }

    pub fn objects(&self) -> &[Group] {
        unsafe { slice::from_raw_parts((*self.mesh).objects as *mut Group, (*self.mesh).object_count as usize) }
    }

    pub fn groups(&self) -> &[Group] {
        unsafe { slice::from_raw_parts((*self.mesh).groups as *mut Group, (*self.mesh).group_count as usize) }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            ffi::fast_obj_destroy(self.mesh);
        }
    }
}
