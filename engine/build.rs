fn main() {
    #[cfg(feature = "ffi")]
    {
        uniffi::generate_scaffolding("src/ffi/mod.rs").unwrap();
    }
}
