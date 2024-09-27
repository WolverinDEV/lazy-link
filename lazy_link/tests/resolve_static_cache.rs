use lazy_link::lazy_link;
use std::{ptr::NonNull, sync::Mutex};

#[lazy_link(resolver = "my_resolver", cache = "static")]
extern "C" {
    fn method_01();
    fn method_02();
}

extern "C" fn fallback_impl() {}

static RESOLVE_REQUESTS: Mutex<Vec<(&'static str, &'static str)>> = Mutex::new(Vec::new());
fn my_resolver(module: Option<&'static str>, name: &'static str) -> NonNull<()> {
    let mut requests = RESOLVE_REQUESTS.lock().unwrap();
    requests.push((module.unwrap_or(""), name));

    NonNull::new(fallback_impl as *mut ()).unwrap()
}

#[test]
fn test_resolve_static_cache() {
    assert_eq!(RESOLVE_REQUESTS.lock().unwrap().as_slice(), []);

    unsafe {
        method_01();
    }
    assert_eq!(
        RESOLVE_REQUESTS.lock().unwrap().as_slice(),
        [("", "method_01")]
    );

    unsafe {
        method_02();
    }
    assert_eq!(
        RESOLVE_REQUESTS.lock().unwrap().as_slice(),
        [("", "method_01"), ("", "method_02")]
    );

    unsafe {
        method_01();
    }
    assert_eq!(
        RESOLVE_REQUESTS.lock().unwrap().as_slice(),
        [("", "method_01"), ("", "method_02")]
    );

    unsafe {
        method_01();
    }
    assert_eq!(
        RESOLVE_REQUESTS.lock().unwrap().as_slice(),
        [("", "method_01"), ("", "method_02"),]
    );
}
