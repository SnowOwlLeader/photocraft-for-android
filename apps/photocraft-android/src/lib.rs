//! Native Android shell for the shared PhotoCraft editor.
#[cfg(target_os = "android")]
mod android;
#[cfg(any(target_os = "android", test))]
mod documents;
#[cfg(any(target_os = "android", test))]
mod file_browser;
#[cfg(any(target_os = "android", test))]
mod pixel_pencil;
#[cfg(any(target_os = "android", test))]
mod smoke;
