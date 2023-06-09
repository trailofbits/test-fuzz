use internal::{
    auto_concretize_enabled,
    dirs::{concretizations_directory_from_target, impl_concretizations_directory_from_target},
};
use std::fs::remove_dir_all;
use testing::{examples, CommandExt};

#[test]
fn auto_concretize() {
    if !auto_concretize_enabled() {
        return;
    }

    test();

    test_fuzz();
}

fn test() {
    let mut mod_path = String::new();

    for i in 1..=4 {
        let test = format!("{mod_path}test");
        let target = format!("{mod_path}target");

        mod_path = format!("{mod_path}auto_concretize_{i}::");

        let impl_concretizations =
            impl_concretizations_directory_from_target("auto_concretize_0", &target);

        remove_dir_all(impl_concretizations).unwrap_or_default();

        let concretizations = concretizations_directory_from_target("auto_concretize_0", &target);

        remove_dir_all(concretizations).unwrap_or_default();

        examples::test("auto_concretize_0", &test)
            .unwrap()
            .logged_assert()
            .success();
    }
}

fn test_fuzz() {
    let mut mod_path = String::new();

    for i in 1..=4 {
        let target = format!("{mod_path}target");

        mod_path = format!("{mod_path}auto_concretize_{i}::");

        examples::test_fuzz("auto_concretize_0", &target)
            .unwrap()
            .args(["--no-run"])
            .logged_assert()
            .success();
    }
}
