#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris"
))]
mod unix;
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris"
))]
pub use self::unix::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
    target_os = "windows"
)))]
mod unsupported;
#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
    target_os = "windows"
)))]
pub use self::unsupported::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};

// Fast hash types for performance-critical paths
pub mod hash {
    #[cfg(feature = "fast-hash")]
    pub use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

    #[cfg(not(feature = "fast-hash"))]
    pub use ahash::{AHashMap as FxHashMap, AHashSet as FxHashSet, AHasher as FxHasher};

    // Always export ahash for general use (better DoS resistance)
    pub use ahash::{AHashMap, AHashSet, AHasher};
}

// String interning for memory optimization
#[cfg(feature = "string-interning")]
pub mod interning {
    use lasso::{Rodeo, Spur};
    use std::sync::Mutex;

    lazy_static::lazy_static! {
        static ref INTERNER: Mutex<Rodeo> = Mutex::new(Rodeo::new());
    }

    pub fn intern(s: &str) -> Spur {
        INTERNER.lock().unwrap().get_or_intern(s)
    }

    pub fn resolve(id: Spur) -> String {
        INTERNER.lock().unwrap().resolve(&id).to_string()
    }

    pub fn try_resolve(id: &Spur) -> Option<String> {
        INTERNER
            .lock()
            .ok()
            .and_then(|i| i.try_resolve(id).map(|s| s.to_string()))
    }
}

#[cfg(not(feature = "string-interning"))]
pub mod interning {
    // Dummy implementation when feature is disabled
    pub type Spur = String;

    #[inline(always)]
    pub fn intern(s: &str) -> Spur {
        s.to_string()
    }

    #[inline(always)]
    pub fn resolve(id: Spur) -> String {
        id
    }

    #[inline(always)]
    pub fn try_resolve(id: &Spur) -> Option<String> {
        Some(id.clone())
    }
}
