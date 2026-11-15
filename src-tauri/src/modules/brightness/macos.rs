use std::ffi::{c_char, c_int, c_void};

type GetBrightness = unsafe extern "C" fn(u32, *mut f32) -> i32;

unsafe extern "C" {
    fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGMainDisplayID() -> u32;
}

pub struct Probe {
    get: Option<GetBrightness>,
}

impl Probe {
    pub fn new() -> Self {
        let get = unsafe {
            let lib = dlopen(c"/System/Library/PrivateFrameworks/DisplayServices.framework/DisplayServices".as_ptr(), 1);
            let sym = if lib.is_null() { std::ptr::null_mut() } else { dlsym(lib, c"DisplayServicesGetBrightness".as_ptr()) };
            (!sym.is_null()).then(|| std::mem::transmute::<*mut c_void, GetBrightness>(sym))
        };
        Self { get }
    }

    pub fn level(&mut self, _tick: u64) -> Option<u8> {
        let get = self.get?;
        let mut v = 0f32;
        (unsafe { get(CGMainDisplayID(), &mut v) } == 0).then(|| (v.clamp(0.0, 1.0) * 100.0).round() as u8)
    }
}
