//! Native Android shell for the shared PhotoCraft editor.
#[cfg(target_os = "android")]
mod android;
#[cfg(any(target_os = "android", test))]
mod smoke;
