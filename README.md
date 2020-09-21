# test-fuzz

## Setup

1. Build the `cargo-test-fuzz` binary and add it to your `PATH`:
```
$ cd path/to/test-fuzz
$ cargo build
$ source env.sh
```
2. Add the following `dependencies` to your crate's `Cargo.toml` file:
```toml
serde = "1.0"
test-fuzz = { path = "path/to/test-fuzz" }
```
3. Add the following `use` declaration to a file or module containing potential fuzz targets:
```rust
use test_fuzz::test_fuzz;
```

## Usage

1. **Find a fuzz target** and precede it with the `test_fuzz` attribute:
```rust
#[test_fuzz]
fn foo(...) {
    ...
}
```

2. **Generate a corpus** by runnig `cargo test`:
```
$ cargo test
```

3. **Fuzz your target** by runnig `cargo test-fuzz`:
```
$ cargo test-fuzz --target foo
```
