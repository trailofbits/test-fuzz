use predicates::prelude::*;
use testing::{CommandExt, fuzzable, retry};

const MAX_TOTAL_TIME: &str = "60";

#[test]
fn fuzz_profile() {
    fuzzable::test("profile", "test")
        .unwrap()
        .logged_assert()
        .success();

    for use_release in [false, true] {
        let mut args = vec![
            "--exit-code",
            "--run-until-crash",
            "--max-total-time",
            MAX_TOTAL_TIME,
        ];
        let code = if use_release {
            args.push("--release");
            1
        } else {
            0
        };
        retry(3, || {
            fuzzable::test_fuzz("profile", "target")
                .unwrap()
                .args(&args)
                .logged_assert()
                .try_code(predicate::eq(code))
        })
        .unwrap();
        if use_release {
            fuzzable::test_fuzz("profile", "target")
                .unwrap()
                .args(["--replay=crashes", "--release"])
                .logged_assert()
                .success()
                .stdout(predicate::str::contains(
                    r#"assertion failed: !x || PROFILE != "release""#,
                ));
        }
    }
}
