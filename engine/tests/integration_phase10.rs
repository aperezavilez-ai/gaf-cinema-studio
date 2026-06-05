//! Phase 10: C ABI available with --features ffi (see ffi/capi.rs tests).

#[test]
fn phase10_capi_module_documented() {
    // Full C ABI tests run with: cargo test --features ffi capi::
    assert!(std::path::Path::new("src/ffi/capi.rs").exists());
}
