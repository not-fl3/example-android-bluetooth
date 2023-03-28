#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod ios;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use ios::*;

// #[cfg(target_os = "macos")]
// pub mod dummy;

// #[cfg(target_os = "macos")]
// pub use dummy::*;
