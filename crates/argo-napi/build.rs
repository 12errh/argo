// Custom build script that skips libnode.dll check on Windows
// napi-build requires libnode.dll on Windows, but we can skip it for cargo check

#[cfg(target_os = "windows")]
fn main() {
    // On Windows, skip napi-build setup during cargo check
    // This allows cargo check to pass without libnode.dll
    // The actual build with napi-rs CLI will handle type generation
}

#[cfg(not(target_os = "windows"))]
fn main() {
    napi_build::setup();
}
