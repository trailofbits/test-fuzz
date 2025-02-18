use predicates::prelude::*;
use testing::{examples, retry, unique_id, CommandExt};

const MAX_TOTAL_TIME: &str = "60";

#[test]
fn fuzz_cast() {
    examples::test("cast", "test")
        .unwrap()
        .logged_assert()
        .success();

    for use_cast_checks in [false, true] {
        let id = unique_id();

        let mut args = vec![
            "--exit-code",
            "--run-until-crash",
            "--max-total-time",
            MAX_TOTAL_TIME,
        ];
        let code = if use_cast_checks {
            args.push("--features=test-fuzz/cast_checks");
            1
        } else {
            0
        };
        args.extend(["--", "-M", &id]);
        retry(3, || {
            examples::test_fuzz("cast", "target")
                .unwrap()
                .args(&args)
                .logged_assert()
                .try_code(predicate::eq(code))
        })
        .unwrap();
        if use_cast_checks {
            examples::test_fuzz("cast", "target")
                .unwrap()
                .args([
                    "--replay=crashes",
                    "--features=test-fuzz/cast_checks",
                    "--",
                    "-M",
                    &id,
                ])
                .logged_assert()
                .success()
                .stdout(predicate::str::contains("invalid cast"));
        }
    }
}
