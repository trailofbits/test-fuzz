WIP

## Tests

Tests in this repository reside in three places:

- `cargo-test-fuzz/tests` - Tests that exercise `cargo-test-fuzz`. Files should be named for the `cargo-test-fuzz` feature or function they exercise.
- `test-fuzz/tests` - Tests that exercise `test-fuzz`, but not `cargo-test-fuzz`. Files should be named for the `test-fuzz` feature or function they exercise.
- `examples/tests` - Targets of the previous two types of tests. Files should be named for the Rust feature, function, library, or trait they exercise.
