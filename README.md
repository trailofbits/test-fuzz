# test-fuzz

## Installation

```
$ cargo install test-fuzz --version '>=0.1.0-alpha'
```

## Usage

1. **Identify a fuzz target** by:
    - Adding the following `dependencies` to the target crate's `Cargo.toml` file:
        ```toml
        serde = "1.0"
        test-fuzz = "0.1.0-alpha"
        ```
    - Adding the following `use` declaration to the target file or module:
        ```rust
        use test_fuzz::test_fuzz;
        ```
    - Preceding the target function with the `test_fuzz` attribute:
        ```rust
        #[test_fuzz]
        fn foo(...) {
            ...
        }
        ```

2. **Generate a corpus** by running `cargo test`:
    ```
    $ cargo test
    ```

3. **Fuzz your target** by running `cargo test-fuzz`:
    ```
    $ cargo test-fuzz --target foo
    ```

## Features

### `test_fuzz`

* **`skip`** - TODO

* **`rename = "name"`** - TODO

### `test_fuzz_impl`

TODO

### `cargo test-fuzz` options

* **`-- <args>...`** - TODO

* **`--list`** - TODO

* **`--no-instrumentation`** - TODO

* **`--no-run`** - TODO

* **`--persistent`** - TODO

* **`--resume`** - TODO

* **`-p, --package = <package>`** - TODO

* **`--target = <target>`** - TODO

## Limitations

* **Global variables** - TODO

* **Type parameters** - TODO
