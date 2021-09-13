#[must_use]
pub fn enabled() -> bool {
    cfg!(feature = "__auto_concretize")
}
