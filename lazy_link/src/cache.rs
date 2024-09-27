use core::{
    cell::UnsafeCell,
    ptr::{
        self,
        NonNull,
    },
    sync::atomic::{
        AtomicBool,
        AtomicPtr,
        Ordering,
    },
};

/// A trait representing different caching strategies for resolved function pointers.
///
/// This trait is implemented by various cache modes, including `"none"`, `"static"`, and `"static-atomic"`.
/// Each implementation defines how the resolved function pointer is stored and accessed, with
/// considerations for race conditions and performance.
pub trait Cache {
    fn resolve(&self, _: impl FnOnce() -> NonNull<()>) -> NonNull<()>;
}

/// Implementation of the `"static"` cache mode.
///
/// This caches the resolved function pointer in a static variable without handling race conditions.
/// As a result, the resolver may be called multiple times in concurrent scenarios.
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

impl Default for StaticCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for StaticCache {
    fn resolve(&self, resolver: impl FnOnce() -> NonNull<()>) -> NonNull<()> {
        let value = unsafe { &mut *self.value.get() };
        *value.get_or_insert_with(resolver)
    }
}

/// Implementation of the `"static-atomic"` cache mode.
///
/// This caches the resolved function pointer in an atomic variable, addressing race conditions.
/// This ensures that the resolver will be called only once, providing thread-safe access to the
/// cached function pointer.
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

impl Default for StaticAtomicCache {
    fn default() -> Self {
        Self::new()
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

/// Do not cache the resolved value.
///
/// The resolver will be called every time the function is accessed,
/// ensuring the most current value is retrieved without any caching.
pub struct NoCache;

impl NoCache {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NoCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for NoCache {
    fn resolve(&self, resolver: impl FnOnce() -> NonNull<()>) -> NonNull<()> {
        resolver()
    }
}
