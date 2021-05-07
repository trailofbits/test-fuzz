# test-fuzz

At a high-level, `test-fuzz` is a convenient front end for [`afl.rs`](https://github.com/rust-fuzz/afl.rs). In more concrete terms, `test-fuzz` is a collection of Rust macros and a Cargo subcommand that automate certain fuzzing-related tasks, most notably:

* generating a fuzzing corpus
* implementing a fuzzing harness

`test-fuzz` accomplishes these (in part) using Rust's testing facilities. For example, to generate a fuzzing corpus, `test-fuzz` records a target's arguments each time it is called during an invocation of `cargo test`. Similarly, `test-fuzz` implements a fuzzing harness as an additional test in a `cargo-test`-generated binary. This tight integration with Rust's testing facilities is what motivates the name **`test`**`-fuzz`.

**Contents**

1. [Installation](#installation)
2. [Usage](#usage)
3. [Components](#components)
    * [`test_fuzz` macro](#test_fuzz-macro)
    * [`test_fuzz_impl` macro](#test_fuzz_impl-macro)
    * [`cargo test-fuzz` command](#cargo-test-fuzz-command)
4. [Environment Variables](#environment-variables)
5. [Limitations](#limitations)
6. [Tips and Tricks](#tips-and-tricks)

## Installation

```
$ cargo install cargo-test-fuzz --version '>=0.1.0-alpha'
```

## Usage

Fuzzing with `test-fuzz` is essentially three steps:*

1. **Identify a fuzz target**:
    - Add the following `dependencies` to the target crate's `Cargo.toml` file:
        ```toml
        serde = "1.0"
        test-fuzz = "0.1.0-alpha"
        ```
    - Precede the target function with the [`test_fuzz`](#test_fuzz-macro) macro:
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

3. **Fuzz your target** by running [`cargo test-fuzz`](#cargo-test-fuzz-command):
    ```
    $ cargo test-fuzz --target foo
    ```

\* Some additional steps may be necessary following a reboot. AFL requires the following commands to be run as root:

* Linux
    ```sh
    echo core >/proc/sys/kernel/core_pattern
    cd /sys/devices/system/cpu
    echo performance | tee cpu*/cpufreq/scaling_governor
    ```

* OSX
    ```sh
    SL=/System/Library; PL=com.apple.ReportCrash
    launchctl unload -w ${SL}/LaunchAgents/${PL}.plist
    sudo launchctl unload -w ${SL}/LaunchDaemons/${PL}.Root.plist
    ```

## Components

### `test_fuzz` macro

Preceding a function with the `test_fuzz` macro indicates that the function is a fuzz target.

The primary effects of the `test_fuzz` macro are:
* Add instrumentation to the target to serialize its arguments and write them to a corpus file each time the target is called. The instrumentation is guarded by `#[cfg(test)]` so that corpus files are generated only when running tests (however, see [`enable_in_production`](#options) below).
* Add a test to read and deserialize arguments from standard input and apply the target to them. The test checks an environment variable, set by [`cargo test-fuzz`](#cargo-test-fuzz-command), so that the test does not block trying to read from standard input during a normal invocation of `cargo test`. The test is enclosed in a module to reduce the likelihood of a name collision. Currently, the name of the module is `target_fuzz`, where `target` is the name of the target (however, see [`rename`](#options) below).

#### Options

* **`enable_in_production`** - Generate corpus files when not running tests, provided the environment variable [`TEST_FUZZ_WRITE`](#environment-variables) is set. The default is to generate corpus files only when running tests, regardless of whether [`TEST_FUZZ_WRITE`](#environment-variables) is set. When running a target from outside its package directory, set [`TEST_FUZZ_MANIFEST_PATH`](#environment-variables) to the path of the package's `Cargo.toml` file.

    **WARNING**: Setting `enable_in_production` could introduce a denial-of-service vector. For example, setting this option for a function that is called many times with different arguments could fill up the disk. The check of [`TEST_FUZZ_WRITE`](#environment-variables) is meant to provide some defense against this possibility. Nonetheless, consider this option carefully before using it.

* **`rename = "name"`** - Treat the target as though its name is `name` when adding a module to the enclosing scope. Expansion of the `test_fuzz` macro adds a module definition to the enclosing scope. Currently, the module is named `target_fuzz`, where `target` is the name of the target. Use of this option causes the module to instead be be named `name_fuzz`. Example:
    ```rust
    #[test_fuzz(rename = "bar")]
    fn foo() {}

    // Without the use of `rename`, a name collision and compile error would occur.
    mod foo_fuzz {}
    ```

* **`skip`** - Disable the use of the `test_fuzz` macro in which `skip` appears.

* **`specialize = "parameters"`** - Use `parameters` as the target's type parameters when fuzzing. Example:
    ```rust
    #[test_fuzz(specialize = "String")]
    fn foo<T: Clone + Debug + Serialize>(x: &T) {
        ...
    }
    ```
    Note: The target's arguments must be serializable for **every** instantiation of its type parameters. But the target's arguments are required to be deserializable only when the target is instantiated with `parameters`.

* **`specialize_impl = "parameters"`** - Use `parameters` as the target's `Self` type parameters when fuzzing. Example:
    ```rust
    #[test_fuzz_impl]
    impl<T: Clone + Debug + Serialize> for Foo {
        #[test_fuzz(specialize_impl = "String")]
        fn bar(&self, x: &T) {
            ...
        }
    }
    ```
    Note: The target's arguments must be serializable for **every** instantiation of its `Self` type parameters. But the target's arguments are required to be deserializable only when the target's `Self` is instantiated with `parameters`.

### `test_fuzz_impl` macro

Whenever the [`test_fuzz`](#test_fuzz-macro) macro is used in an `impl` block,
the `impl` must be preceded with the `test_fuzz_impl` macro. Example:

```rust
#[test_fuzz_impl]
impl Foo {
    #[test_fuzz]
    fn bar(&self, x: &str) {
        ...
    }
}
```

The reason for this requirement is as follows. Expansion of the [`test_fuzz`](#test_fuzz-macro) macro adds a module definition to the enclosing scope. However, a module definition cannot appear inside an `impl` block. Preceding the `impl` with the `test_fuzz_impl` macro causes the module to be added outside the `impl` block.

If you see an error like the following, it likely means a use of the `test_fuzz_impl` macro is missing:
```
error: module is not supported in `trait`s or `impl`s
```

`test_fuzz_impl` currently has no options.

### `cargo test-fuzz` command

The `cargo test-fuzz` command is used to interact with fuzz targets, and to manipulate their corpora, crashes, hangs, and work queues. Example invocations include:

1. List fuzz targets
    ```
    cargo test-fuzz --list
    ```

2. Display target `foo`'s corpus
    ```
    cargo test-fuzz --target foo --display-corpus
    ```

3. Fuzz target `foo`
    ```
    cargo test-fuzz --target foo
    ```

4. Replay crashes found for target `foo`
    ```
    cargo test-fuzz --target foo --replay-crashes
    ```

#### Flags

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

#### Options

* **`-- <args>...`** - Arguments for the fuzzer

* **`-p, --package = <package>`** - Package containing fuzz target

* **`--target = <target>`** - String that fuzz target's name must contain

* **`--timeout <timeout>`** - Number of milliseconds to consider a hang when fuzzing or replaying (equivalent to `-- -t <timeout>` when fuzzing)

## Environment Variables

* **`TEST_FUZZ_LOG`** - During macro expansion, write instrumented fuzz targets and their associated module definitions to standard output. This can be useful for debugging.

* **`TEST_FUZZ_MANIFEST_PATH`** - When running a target from outside its package directory, find the package's `Cargo.toml` file at this location. One may need to set this environment variable when [`enable_in_production`](#options) is used.

* **`TEST_FUZZ_WRITE`** - Generate corpus files when not running tests for those targets for which [`enable_in_production`](#options) is set.

## Limitations

* **Clonable arguments** - A target's arguments must implement the [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) trait. The reason for this requirement is that the arguments are needed in two places: in a `test-fuzz`-internal function that writes corpus files, and in the body of the target function. To resolve this conflict, the arguments are cloned before being passed to the former.

* **Serializable / deserializable arguments** - In general, a target's arguments must implement the [`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html) and [`serde::Deserialize`](https://docs.serde.rs/serde/trait.Deserialize.html) traits, e.g., by [deriving them](https://serde.rs/derive.html). We say "in general" because `test-fuzz` knows how to handle certain special cases that wouldn't normally be serializable / deserializable. For example, an argument of type `&str` is converted to `String` when serializing, and back to a `&str` when deserializing. See also [`specialize` and `specialize_impl`](#options) above.

* **Global variables** - The fuzzing harnesses that `test-fuzz` generates do not initialize global variables. No general purpose solution for this problem currently exists. So, to fuzz a function that relies on global variables using `test-fuzz`, ad-hoc methods must be used.

## Tips and tricks

* `#[cfg(test)]` [is not enabled](https://github.com/rust-lang/rust/issues/45599#issuecomment-460488107) for integration tests. If your target is tested only by integration tests, then consider using [`enable_in_production`](#options) and [`TEST_FUZZ_WRITE`](#environment-variables) to generate a corpus. (Note the warning accompanying [`enable_in_production`](#options), however.)

* Rust [won't allow you to](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types) implement `serde::Serializable` for other repositories' types. But you may be able to [patch](https://doc.rust-lang.org/edition-guide/rust-2018/cargo-and-crates-io/replacing-dependencies-with-patch.html) other repositories to make their types serializeble. Also, [`cargo-clone`](https://github.com/JanLikar/cargo-clone) can be useful for grabbing dependencies' repositories.
