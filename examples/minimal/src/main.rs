use std::ptr::NonNull;

use lazy_link::lazy_link;

#[lazy_link(resolver = "resolve_externals")]
extern "C" {
    fn external_add(v1: u8, v2: u8) -> u8;
}

extern "C" fn impl_external_add(v1: u8, v2: u8) -> u8 {
    v1.wrapping_add(v2)
}

fn resolve_externals(_module: Option<&str>, name: &str) -> NonNull<()> {
    assert_eq!(name, "external_add");

    NonNull::new(impl_external_add as *mut ()).unwrap()
}

fn main() {
    let answer = unsafe { external_add(22, 20) };
    println!("The answer is {}", answer);
}
