use internal::dirs::{generic_args_directory_from_target, impl_generic_args_directory_from_target};
use std::fs::remove_dir_all;
use testing::{examples, CommandExt};

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn generic() {
    let impl_expected = ["generic::Bar", "generic::Foo"];
    let expected = ["generic::Baz<generic::Bar>", "generic::Baz<generic::Foo>"];
    test(
        "generic",
        "test_bound",
        "target_bound",
        &impl_expected,
        &expected,
    );
    test(
        "generic",
        "test_where_clause",
        "target_where_clause",
        &impl_expected,
        &expected,
    );
    test(
        "generic",
        "test_only_generic_args",
        "target_only_generic_args",
        &impl_expected,
        &expected,
    );
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn unserde() {
    let impl_expected = [""];
    let expected = ["unserde::Struct"];
    test("unserde", "test", "target", &impl_expected, &expected);
    test(
        "unserde",
        "test_in_production",
        "target_in_production",
        &impl_expected,
        &expected,
    );
}

fn test(krate: &str, test: &str, target: &str, impl_expected: &[&str], expected: &[&str]) {
    let impl_generic_args = impl_generic_args_directory_from_target(krate, target);

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(impl_generic_args).unwrap_or_default();

    let generic_args = generic_args_directory_from_target(krate, target);

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(generic_args).unwrap_or_default();

    examples::test(krate, test)
        .unwrap()
        .logged_assert()
        .success();

    for (option, expected) in &[
        ("--display=impl-generic-args", impl_expected),
        ("--display=generic-args", expected),
    ] {
        let assert = &examples::test_fuzz(krate, target)
            .unwrap()
            .args([option])
            .logged_assert()
            .success();

        let mut actual = std::str::from_utf8(&assert.get_output().stdout)
            .unwrap()
            .lines()
            .map(|s| {
                let n = s.find(": ").unwrap();
                s[n + 2..].to_owned()
            })
            .collect::<Vec<_>>();

        actual.sort();

        assert_eq!(expected, &actual);
    }
}
