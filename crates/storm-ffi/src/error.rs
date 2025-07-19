// File: crates/storm-ffi/src/error.rs
// FFI error handling

/// FFI error codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Success = 0,
    InvalidHandle = -1,
    InvalidParameter = -2,
    InitializationFailed = -3,
    NetworkError = -4,
    GenericError = -99,
}

impl From<anyhow::Error> for ErrorCode {
    fn from(_: anyhow::Error) -> Self {
        ErrorCode::GenericError
    }
}