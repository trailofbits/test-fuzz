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
    - Precede the target function with the `test_fuzz` attribute:
        ```rust
        #[test_fuzz::test_fuzz]
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

* **`-- <args>...`** - Arguments for the fuzzer

* **`--backtrace`** - Display backtraces

* **`--consolidate`** - Move one target's crashes and work queue to its corpus; to consolidate all targets, use `--consolidate-all`

* **`--display-corpus`** - Display corpus using uninstrumented fuzz target; to display with instrumentation, use `--display-corpus-instrumented`

* **`--display-crashes`** - Display crashes

* **`--display-queue`** - Display work queue

* **`--exact`** - Target name is an exact name rather than a substring

* **`--list`** - List fuzz targets

* **`--no-instrumentation`** - Compile without instrumentation (for testing build process)

* **`--no-run`** - Compile, but don't fuzz

* **`--no-ui`** - Disable user interface

* **`--persistent`** - Enable persistent mode fuzzing

* **`--pretty-print`** - Pretty-print debug output when displaying/replaying

* **`--replay-corpus`** - Replay corpus using uninstrumented fuzz target; to replay with instrumentation, use `--replay-corpus-instrumented`

* **`--replay-crashes`** - Replay crashes

* **`--replay-queue`** - Replay work queue

* **`--reset`** - Clear fuzzing data for one target, but leave corpus intact; to reset all targets, use `--reset-all`

* **`--resume`** - Resume target's last fuzzing session

* **`--run-until-crash`** - Stop fuzzing once a crash is found

* **`-p, --package = <package>`** - Package containing fuzz target

* **`--target = <target>`** - String that fuzz target's name must contain

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
