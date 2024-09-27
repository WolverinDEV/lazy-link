use std::{
    ptr::NonNull,
    sync::Mutex,
};

use lazy_link::lazy_link;

#[lazy_link(resolver = "my_resolver", module = "hello_module", obfuscate = false)]
extern "C" {
    fn method_01();
}

extern "C" fn fallback_impl() {}

static RESOLVE_REQUESTS: Mutex<Vec<(&'static str, &'static str)>> = Mutex::new(Vec::new());
fn my_resolver(module: Option<&'static str>, name: &'static str) -> NonNull<()> {
    let mut requests = RESOLVE_REQUESTS.lock().unwrap();
    requests.push((module.unwrap_or(""), name));

    NonNull::new(fallback_impl as *mut ()).unwrap()
}

#[test]
fn test_resolve_nocache() {
    assert_eq!(RESOLVE_REQUESTS.lock().unwrap().as_slice(), []);

    unsafe {
        method_01();
    }
    assert_eq!(
        RESOLVE_REQUESTS.lock().unwrap().as_slice(),
        [("hello_module", "method_01")]
    );
}
