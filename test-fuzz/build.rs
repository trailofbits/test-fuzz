#[cfg(not(any(feature = "serde_bincode", feature = "serde_cbor",)))]
println!("cargo:rustc-cfg=serde_default");
