#[cfg(test)]
mod tests {
    use internal::{
        auto_concretize_enabled,
        dirs::{concretizations_directory_from_target, impl_concretizations_directory_from_target},
    };
    use std::fs::remove_dir_all;
    use testing::examples;

    #[test]
    fn auto_concretize() {
        if !auto_concretize_enabled() {
            return;
        }

        let impl_concretizations = impl_concretizations_directory_from_target(
            "auto_concretize",
            "auto_concretize::target",
        );

        remove_dir_all(&impl_concretizations).unwrap_or_default();

        let concretizations =
            concretizations_directory_from_target("auto_concretize", "auto_concretize::target");

        remove_dir_all(&concretizations).unwrap_or_default();

        examples::test("auto_concretize", "auto_concretize::test")
            .unwrap()
            .assert()
            .success();

        examples::test_fuzz("auto_concretize", "auto_concretize::target")
            .unwrap()
            .args(&["--no-run"])
            .assert()
            .success();
    }
}
