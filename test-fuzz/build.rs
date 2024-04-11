fn main() {
    #[cfg(not(any(
        feature = "serde_bincode",
        feature = "serde_cbor",
        feature = "serde_cbor4ii",
        feature = "serde_postcard"
    )))]
    println!("cargo:rustc-cfg=serde_default");
}
