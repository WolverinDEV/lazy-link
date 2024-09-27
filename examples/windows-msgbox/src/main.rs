use std::{ffi::c_char, ptr::NonNull};

use lazy_link::lazy_link;
use libloading::Library;

#[lazy_link(module = "User32.dll", resolver = "win32_resolve")]
extern "C" {
    #[allow(non_snake_case)]
    fn MessageBoxA(hWnd: u32, lpText: *const c_char, lpCaption: *const c_char, uType: u32) -> u8;
}

unsafe fn win32_resolve(module: Option<&'static str>, name: &'static str) -> NonNull<()> {
    let module = module.expect("a module to be specified");
    let library = Library::new(module).expect("failed to load target library");

    let symbol_cname = format!("{}\0", name);
    let symbol = library
        .get::<*mut ()>(symbol_cname.as_bytes())
        .expect(&format!("could not resolve symbol {}", symbol_cname));

    NonNull::new_unchecked(symbol.try_as_raw_ptr().unwrap() as *mut ())
}

fn main() {
    unsafe {
        MessageBoxA(
            0,
            "Dummy content\0".as_ptr() as *const c_char,
            "Lazy imports :)\0".as_ptr() as *const c_char,
            0x00,
        );
    }
}
