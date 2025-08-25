mod auto_generate;
mod ci;
mod conversion;
mod default;
mod in_production;
mod link;
mod rename;
mod self_ty_in_mod_name;
mod serde_format;
mod test_fuzz_log;
mod versions;

#[ctor::ctor]
fn initialize() {
    unsafe {
        std::env::set_var("CARGO_TERM_COLOR", "never");
    }
}
