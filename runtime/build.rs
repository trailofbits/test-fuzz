fn main() {
    #[cfg(not(any(
        feature = "__fuzzer_aflplusplus",
        feature = "__fuzzer_aflplusplus_persistent",
        feature = "__fuzzer_libfuzzer",
    )))]
    println!("cargo:rustc-cfg=fuzzer_default");

    #[cfg(not(any(
        feature = "__serde_bincode",
        feature = "__serde_cbor",
        feature = "__serde_cbor4ii"
    )))]
    println!("cargo:rustc-cfg=serde_default");
}
