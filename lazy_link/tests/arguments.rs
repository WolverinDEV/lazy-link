use std::{
    ptr::NonNull,
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};

use lazy_link::lazy_link;

#[lazy_link(resolver = "my_resolver")]
extern "C" {
    fn method_01(_: u8);
    fn method_02(arg1: u8, arg2: u8);
}

extern "C" fn fallback_impl_01(arg: u8) {
    IMPL_01_CALLED.store(true, Ordering::Relaxed);
    assert_eq!(arg, 0x67);
}

extern "C" fn fallback_impl_02(arg1: u8, arg2: u8) {
    IMPL_02_CALLED.store(true, Ordering::Relaxed);
    assert_eq!(arg1, 0x77);
    assert_eq!(arg2, 0x34);
}

static IMPL_01_CALLED: AtomicBool = AtomicBool::new(false);
static IMPL_02_CALLED: AtomicBool = AtomicBool::new(false);

fn my_resolver(_module: Option<&str>, name: &str) -> NonNull<()> {
    NonNull::new(match name {
        "method_01" => fallback_impl_01 as *mut (),
        "method_02" => fallback_impl_02 as *mut (),
        _ => unreachable!(),
    })
    .unwrap()
}

#[test]
fn test_resolve_nocache() {
    unsafe {
        method_01(0x67);
        method_02(0x77, 0x34);
    }

    assert!(IMPL_01_CALLED.load(Ordering::Relaxed));
    assert!(IMPL_02_CALLED.load(Ordering::Relaxed));
}
