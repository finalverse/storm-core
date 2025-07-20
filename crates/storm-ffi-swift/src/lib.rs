// File: storm-core/crates/storm-ffi-swift/src/lib.rs
// Description: Swift-specific FFI bindings for StormCore
// Provides C-compatible interface for iOS/macOS applications

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

/// Add function for basic testing
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// Initialize StormCore for Swift/iOS
#[no_mangle]
pub extern "C" fn storm_swift_init() -> *mut c_void {
    // Initialize storm core for Swift
    // Return opaque handle
    std::ptr::null_mut()
}

/// Shutdown StormCore for Swift/iOS
#[no_mangle]
pub extern "C" fn storm_swift_shutdown(handle: *mut c_void) -> i32 {
    if handle.is_null() {
        return -1;
    }
    // Cleanup logic here
    0 // Success
}

/// Get version string for Swift
#[no_mangle]
pub extern "C" fn storm_swift_get_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

/// Create entity for Swift
#[no_mangle]
pub extern "C" fn storm_swift_create_entity(handle: *mut c_void) -> u64 {
    if handle.is_null() {
        return 0;
    }
    // Return dummy entity ID for now
    1
}

/// Set entity position for Swift
#[no_mangle]
pub extern "C" fn storm_swift_set_entity_position(
    handle: *mut c_void,
    entity_id: u64,
    x: f32,
    y: f32,
    z: f32,
) -> i32 {
    if handle.is_null() {
        return -1;
    }
    // Position setting logic here
    0 // Success
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
    fn test_swift_init() {
        let handle = storm_swift_init();
        // For now, should return null as it's not implemented
        assert!(handle.is_null());
    }

    #[test]
    fn test_version() {
        let version_ptr = storm_swift_get_version();
        assert!(!version_ptr.is_null());

        unsafe {
            let version = CStr::from_ptr(version_ptr);
            let version_str = version.to_str().unwrap();
            assert!(!version_str.is_empty());
        }
    }

    #[test]
    fn test_entity_creation() {
        let handle = storm_swift_init();
        let entity_id = storm_swift_create_entity(handle);
        // Should return 0 for null handle
        assert_eq!(entity_id, 0);
    }
}