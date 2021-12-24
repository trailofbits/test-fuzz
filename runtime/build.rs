fn main() {
    #[cfg(not(any(
        feature = "__serde_bincode",
        feature = "__serde_cbor",
        feature = "__serde_cbor4ii"
    )))]
    println!("cargo:rustc-cfg=serde_default");
}
