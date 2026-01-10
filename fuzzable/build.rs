use std::{
    env::{var, var_os},
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
};

fn main() {
    let profile = var("PROFILE").unwrap();
    let out_dir = var_os("OUT_DIR").unwrap();
    let path_buf = PathBuf::from(out_dir).join("profile.rs");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path_buf)
        .unwrap();
    writeln!(file, r#"const PROFILE: &str = "{profile}";"#).unwrap();
}
