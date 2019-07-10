use capnp::message::TypedReader;
use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_char, c_void};

pub type AllocFn = extern "C" fn(*mut c_void, usize) -> *mut u8;
pub type LoadResourceFn =
    extern "C" fn(*mut c_void, *const c_char, *mut *const u8, *mut usize) -> bool;
pub type ReleaseResourceFn = extern "C" fn(*mut c_void, *const c_char) -> bool;

#[derive(Debug)]
#[repr(C)]
pub struct PipelineInterface {
    pub data: *mut c_void,
    pub alloc_fn: AllocFn,
    pub load_resource_fn: LoadResourceFn,
    // pub release_resource_fn: ReleaseResourceFn,
}

impl Drop for PipelineInterface {
    fn drop(&mut self) {
        println!("drop");
    }
}
unsafe impl Send for PipelineInterface {}
unsafe impl Sync for PipelineInterface {}

impl PipelineInterface {
    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        let result = (self.alloc_fn)(self.data, size);
        if result == std::ptr::null_mut() {
            return None;
        }
        Some(result)
    }

    pub fn load_resource(&self, name: &str) -> Option<PipelineResource> {
        let cstr = CString::new(name).unwrap();
        let mut data: *const u8 = std::ptr::null_mut();
        let mut data_size: usize = 0;
        let result = (self.load_resource_fn)(self.data, cstr.as_ptr(), &mut data, &mut data_size);
        if !result {
            return None;
        }

        Some(PipelineResource {
            name: name.into(),
            data,
            data_size,
        })
    }
}

#[derive(Debug)]
pub struct PipelineResource {
    name: String,
    data: *const u8,
    data_size: usize,
}

impl PipelineResource {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data
    }

    pub fn size(&self) -> usize {
        self.data_size
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.data_size) }
    }
}

impl Drop for PipelineResource {
    fn drop(&mut self) {
        // TODO: release
    }
}

pub static mut PIPELINE_INTERFACE: Option<*const PipelineInterface> = None;

/// To be called by the pipeline module to allocate memory needed for large chunks of data
pub fn allocate(size: usize) -> Option<*mut u8> {
    unsafe {
        if let Some(interface) = PIPELINE_INTERFACE {
            println!(
                "interface {:?} {:?} size {}",
                interface,
                (*interface).data,
                size
            );
            (*interface).alloc(size)
        } else {
            None
        }
    }
}

/// To be called by the pipeline module's pipeline_init function to initialize the SDK
pub fn initialize(interface: *const PipelineInterface) -> bool {
    unsafe {
        println!("initialize {:?} {:?}", interface, (*interface).data,);
        PIPELINE_INTERFACE = Some(interface);
    }
    true
}

pub fn load_resource(name: &str) -> Option<PipelineResource> {
    unsafe {
        if let Some(interface) = PIPELINE_INTERFACE {
            (*interface).load_resource(name)
        } else {
            None
        }
    }
}
