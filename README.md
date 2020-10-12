# test-fuzz

## Installation

```
$ cargo install cargo-test-fuzz --version '>=0.1.0-alpha'
```

## Usage

1. **Identify a fuzz target**:
    - Add the following `dependencies` to the target crate's `Cargo.toml` file:
        ```toml
        serde = "1.0"
        test-fuzz = "0.1.0-alpha"
        ```
    - Add the following `use` declaration to the target file or module:
        ```rust
        use test_fuzz::test_fuzz;
        ```
    - Precede the target function with the `test_fuzz` attribute:
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

## Components

## `test_fuzz` attribute

TODO

### Options

* **`force`** - TODO

* **`rename = "name"`** - TODO

* **`skip`** - TODO

## `test_fuzz_impl` attribute

TODO

`test_fuzz_impl` currently has no options.

## `cargo test-fuzz` command

TODO

### Options

* **`-- <args>...`** - TODO

* **`--backtrace`** - TODO

* **`--display-corpus`** - TODO

* **`--display-crashes`** - TODO

* **`--display-queue`** - TODO

* **`--exact`** - TODO

* **`--list`** - TODO

* **`--no-instrumentation`** - TODO

* **`--no-run`** - TODO

* **`--no-ui`** - TODO

* **`--persistent`** - TODO

* **`--pretty-print`** - TODO

* **`--replay-corpus`** - TODO

* **`--replay-crashes`** - TODO

* **`--replay-queue`** - TODO

* **`--resume`** - TODO

* **`-p, --package = <package>`** - TODO

* **`--target = <target>`** - TODO

## `test_fuzz` crate

TODO

### Features

* **`logging`** - TODO

* **`persistent`** - TODO

## Limitations

* **Clonable arguments** - TODO

* **Seralizable arguments** - TODO

* **Global variables** - TODO

* **Type parameters** - TODO
