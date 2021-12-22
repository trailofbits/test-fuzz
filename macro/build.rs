fn main() {
    #[cfg(not(any(feature = "__serde_bincode", feature = "__serde_cbor",)))]
    println!("cargo:rustc-cfg=serde_default");
}
