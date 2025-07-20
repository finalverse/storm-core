// File: storm-core/crates/storm-ffi-kotlin/src/lib.rs
// Description: Kotlin-specific FFI bindings for StormCore
// Provides JNI-compatible interface for Android applications

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

/// Add function for basic testing
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// Initialize StormCore for Kotlin/Android
#[no_mangle]
pub extern "C" fn storm_kotlin_init() -> *mut c_void {
    // Initialize storm core for Kotlin
    // Return opaque handle
    std::ptr::null_mut()
}

/// Shutdown StormCore for Kotlin/Android
#[no_mangle]
pub extern "C" fn storm_kotlin_shutdown(handle: *mut c_void) -> i32 {
    if handle.is_null() {
        return -1;
    }
    // Cleanup logic here
    0 // Success
}

/// Get version string for Kotlin
#[no_mangle]
pub extern "C" fn storm_kotlin_get_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_kotlin_init() {
        let handle = storm_kotlin_init();
        // For now, should return null as it's not implemented
        assert!(handle.is_null());
    }

    #[test]
    fn test_version() {
        let version_ptr = storm_kotlin_get_version();
        assert!(!version_ptr.is_null());

        unsafe {
            let version = CStr::from_ptr(version_ptr);
            let version_str = version.to_str().unwrap();
            assert!(!version_str.is_empty());
        }
    }
}