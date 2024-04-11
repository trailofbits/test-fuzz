fn main() {
    #[cfg(not(any(
        feature = "__serde_bincode",
        feature = "__serde_cbor",
        feature = "__serde_cbor4ii",
        feature = "__serde_postcard"
    )))]
    println!("cargo:rustc-cfg=serde_default");
}
