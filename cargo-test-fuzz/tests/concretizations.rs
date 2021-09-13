use internal::{
    dirs::{concretizations_directory_from_target, impl_concretizations_directory_from_target},
    examples,
};
use std::fs::remove_dir_all;

#[test]
fn generic() {
    let impl_expected = ["generic::generic::Bar", "generic::generic::Foo"];
    let expected = [
        "generic::generic::Baz<generic::generic::Bar>",
        "generic::generic::Baz<generic::generic::Foo>",
    ];
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
        "test_only_concretizations",
        "target_only_concretizations",
        &impl_expected,
        &expected,
    );
}

#[test]
fn unserde() {
    let impl_expected = [""];
    let expected = ["unserde::unserde::Struct"];
    test("unserde", "test", "target", &impl_expected, &expected);
    test(
        "unserde",
        "test_in_production",
        "target_in_production",
        &impl_expected,
        &expected,
    );
}

fn test(name: &str, test: &str, target: &str, impl_expected: &[&str], expected: &[&str]) {
    let impl_concretizations = impl_concretizations_directory_from_target(name, target);

    remove_dir_all(&impl_concretizations).unwrap_or_default();

    let concretizations = concretizations_directory_from_target(name, target);

    remove_dir_all(&concretizations).unwrap_or_default();

    examples::test(name, &format!("{}::{}", name, test))
        .unwrap()
        .assert()
        .success();

    for (option, expected) in &[
        ("--display-impl-concretizations", impl_expected),
        ("--display-concretizations", expected),
    ] {
        let assert = &examples::test_fuzz(name, &format!("{}::{}", name, target))
            .unwrap()
            .args(&[option])
            .assert()
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
