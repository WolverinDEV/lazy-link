use core::{
    cell::UnsafeCell,
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

pub trait Cache {
    fn resolve(&self, _: impl FnOnce() -> NonNull<()>) -> NonNull<()>;
}

/// Implementation of the "static" cache mode.  
///  
/// This will cache the resolved function pointer in a static variable
/// without any consideration of race conditions. This may result in the resolver being called
/// multiple times.
pub struct StaticCache {
    value: UnsafeCell<Option<NonNull<()>>>,
}

/// As outlined within the considerations when using this cache,
/// race conditions are deliberately allowed.
unsafe impl Sync for StaticCache {}

impl StaticCache {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(None),
        }
    }
}

impl Cache for StaticCache {
    fn resolve(&self, resolver: impl FnOnce() -> NonNull<()>) -> NonNull<()> {
        let value = unsafe { &mut *self.value.get() };
        value.get_or_insert_with(resolver).clone()
    }
}

/// Implementation of the "static-atomic" cache mode.  
///  
/// This will cache the resolved function pointer in an atomic variable
/// with considerations regarding of race conditions. This ensures that the resolver will only
/// be called once.
pub struct StaticAtomicCache {
    value: AtomicPtr<()>,
    resolve_lock: AtomicBool,
}

impl StaticAtomicCache {
    pub const fn new() -> Self {
        Self {
            value: AtomicPtr::new(ptr::null_mut()),
            resolve_lock: AtomicBool::new(false),
        }
    }
}

impl Cache for StaticAtomicCache {
    fn resolve(&self, resolver: impl FnOnce() -> NonNull<()>) -> NonNull<()> {
        loop {
            let value = self.value.load(Ordering::Relaxed);
            if let Some(value) = NonNull::new(value) {
                return value;
            }

            if self
                .resolve_lock
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                /* The value is already getting resolved. */
                continue;
            }

            let result = resolver();
            self.value.store(result.as_ptr(), Ordering::Relaxed);
            return result;
        }
    }
}

pub struct NoCache;

impl NoCache {
    pub const fn new() -> Self {
        Self
    }
}

impl Cache for NoCache {
    fn resolve(&self, resolver: impl FnOnce() -> NonNull<()>) -> NonNull<()> {
        resolver()
    }
}
