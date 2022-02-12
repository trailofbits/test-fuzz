# test-fuzz

At a high-level, `test-fuzz` is a convenient front end for [`afl.rs`](https://github.com/rust-fuzz/afl.rs). In more concrete terms, `test-fuzz` is a collection of Rust macros and a Cargo subcommand that automate certain fuzzing-related tasks, most notably:

- generating a fuzzing corpus
- implementing a fuzzing harness

`test-fuzz` accomplishes these (in part) using Rust's testing facilities. For example, to generate a fuzzing corpus, `test-fuzz` records a target's arguments each time it is called during an invocation of `cargo test`. Similarly, `test-fuzz` implements a fuzzing harness as an additional test in a `cargo-test`-generated binary. This tight integration with Rust's testing facilities is what motivates the name **`test`**`-fuzz`.

**Contents**

1. [Installation](#installation)
2. [Usage](#usage)
3. [Components](#components)
   - [`test_fuzz` macro](#test_fuzz-macro)
   - [`test_fuzz_impl` macro](#test_fuzz_impl-macro)
   - [`cargo test-fuzz` command](#cargo-test-fuzz-command)
   - [Convenience macros](#convenience-macros)
4. [`test-fuzz` package features](#test-fuzz-package-features)
5. [Auto-generated corpus files](#auto-generated-corpus-files)
6. [Environment variables](#environment-variables)
7. [Limitations](#limitations)
8. [Tips and tricks](#tips-and-tricks)

## Installation

Install `cargo-test-fuzz` and [`afl.rs`](https://github.com/rust-fuzz/afl.rs) with the following command:

```sh
cargo install cargo-test-fuzz afl
```

## Usage

Fuzzing with `test-fuzz` is essentially three steps:\*

1. **Identify a fuzz target**:

   - Add the following `dependencies` to the target crate's `Cargo.toml` file:
     ```toml
     serde = "1.0"
     test-fuzz = "2.0.2"
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
   $ cargo test-fuzz foo
   ```

\* Some additional steps may be necessary following a reboot. AFL requires the following commands to be run as root:

- Linux

  ```sh
  echo core >/proc/sys/kernel/core_pattern
  cd /sys/devices/system/cpu
  echo performance | tee cpu*/cpufreq/scaling_governor
  ```

- OSX
  ```sh
  SL=/System/Library; PL=com.apple.ReportCrash
  launchctl unload -w ${SL}/LaunchAgents/${PL}.plist
  sudo launchctl unload -w ${SL}/LaunchDaemons/${PL}.Root.plist
  ```

## Components

### `test_fuzz` macro

Preceding a function with the `test_fuzz` macro indicates that the function is a fuzz target.

The primary effects of the `test_fuzz` macro are:

- Add instrumentation to the target to serialize its arguments and write them to a corpus file each time the target is called. The instrumentation is guarded by `#[cfg(test)]` so that corpus files are generated only when running tests (however, see [`enable_in_production`](#arguments) below).
- Add a test to read and deserialize arguments from standard input and apply the target to them. The test checks an environment variable, set by [`cargo test-fuzz`](#cargo-test-fuzz-command), so that the test does not block trying to read from standard input during a normal invocation of `cargo test`. The test is enclosed in a module to reduce the likelihood of a name collision. Currently, the name of the module is `target_fuzz`, where `target` is the name of the target (however, see [`rename`](#arguments) below).

#### Arguments

- **`bounds = "where_predicates"`** - Impose `where_predicates` (e.g., trait bounds) on the struct used to serialize/deserialize arguments. This may be necessary, e.g., if a target's argument type is an associated type. For an example, see [associated_type.rs](examples/tests/associated_type.rs#L27) in this repository.

- **`concretize = "parameters"`** - Use `parameters` as the target's type parameters when fuzzing. Example:

  ```rust
  #[test_fuzz(concretize = "String")]
  fn foo<T: Clone + Debug + Serialize>(x: &T) {
      ...
  }
  ```

  Note: The target's arguments must be serializable for **every** instantiation of its type parameters. But the target's arguments are required to be deserializable only when the target is instantiated with `parameters`.

- **`concretize_impl = "parameters"`** - Use `parameters` as the target's `Self` type parameters when fuzzing. Example:

  ```rust
  #[test_fuzz_impl]
  impl<T: Clone + Debug + Serialize> for Foo {
      #[test_fuzz(concretize_impl = "String")]
      fn bar(&self, x: &T) {
          ...
      }
  }
  ```

  Note: The target's arguments must be serializable for **every** instantiation of its `Self` type parameters. But the target's arguments are required to be deserializable only when the target's `Self` is instantiated with `parameters`.

- **`convert = "X, Y"`** - When serializing the target's arguments, convert values of type `X` to type `Y` using `Y`'s implementation of `From<X>`. When deserializing, convert those values back to type `X` using `Y`'s implementation of the non-standard trait `test_fuzz::Into<X>`.

  That is, use of `convert = "X, Y"` should be accompanied by the following implementations:

  ```rust
  impl From<X> for Y {
      fn from(x: X) -> Self {
          ...
      }
  }

  impl test_fuzz::Into<X> for Y {
      fn into(self) -> X {
          ...
      }
  }
  ```

  The definition of `test_fuzz::Into` is identical to that of [`std::convert::Into`](https://doc.rust-lang.org/std/convert/trait.Into.html). The reason for using a non-standard trait is to avoid conflicts that could arise from blanket implementations of standard traits.

- **`enable_in_production`** - Generate corpus files when not running tests, provided the environment variable [`TEST_FUZZ_WRITE`](#environment-variables) is set. The default is to generate corpus files only when running tests, regardless of whether [`TEST_FUZZ_WRITE`](#environment-variables) is set. When running a target from outside its package directory, set [`TEST_FUZZ_MANIFEST_PATH`](#environment-variables) to the path of the package's `Cargo.toml` file.

  **WARNING**: Setting `enable_in_production` could introduce a denial-of-service vector. For example, setting this option for a function that is called many times with different arguments could fill up the disk. The check of [`TEST_FUZZ_WRITE`](#environment-variables) is meant to provide some defense against this possibility. Nonetheless, consider this option carefully before using it.

- **`execute_with = "function"`** - Rather than call the target directly:

  - construct a closure of type `FnOnce() -> R`, where `R` is the target's return type, so that calling the closure calls the target;
  - call `function` with the closure.

  Calling the target in this way allows `function` to set up the call's environment. This can be useful, e.g., for fuzzing [Substrate externalities](https://substrate.dev/docs/en/knowledgebase/runtime/tests#mock-runtime-storage).

- **`no_auto_generate`** - Do not try to [auto-generate corpus files](#auto-generated-corpus-files) for the target.

- **`only_concretizations`** - Record the target's concretizations when running tests, but do not generate corpus files and do not implement a fuzzing harness. This can be useful when the target is a generic function, but it is unclear what type parameters should be used for fuzzing.

  The intended workflow is: enable `only_concretizations`, then run `cargo test` followed by `cargo test-fuzz --display-concretizations`. One of the resulting concretizations might be usable as `concretize`'s `parameters`. Similarly, a concretization resulting from `cargo test-fuzz --display-imply-concretizations` might be usable as `concretize_impl`'s `parameters`.

  Note, however, that just because a target was concretized with certain parameters during tests, it does not imply the target's arguments are serializable/deserializable when so concretized. The results of `--display-concretizations`/`--display-impl-concretizations` are merely suggestive.

- **`rename = "name"`** - Treat the target as though its name is `name` when adding a module to the enclosing scope. Expansion of the `test_fuzz` macro adds a module definition to the enclosing scope. Currently, the module is named `target_fuzz`, where `target` is the name of the target. Use of this option causes the module to instead be be named `name_fuzz`. Example:

  ```rust
  #[test_fuzz(rename = "bar")]
  fn foo() {}

  // Without the use of `rename`, a name collision and compile error would result.
  mod foo_fuzz {}
  ```

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
   cargo test-fuzz foo --display-corpus
   ```

3. Fuzz target `foo`

   ```
   cargo test-fuzz foo
   ```

4. Replay crashes found for target `foo`
   ```
   cargo test-fuzz foo --replay-crashes
   ```

#### Usage

```
    cargo test-fuzz [OPTIONS] [TARGETNAME] [-- <args>...]
```

#### Args

```
    <TARGETNAME>    String that fuzz target's name must contain
    <args>...       Arguments for the fuzzer
```

#### Options

```
        --backtrace
            Display backtraces

        --consolidate
            Move one target's crashes, hangs, and work queue to its corpus; to consolidate all
            targets, use --consolidate-all

        --display-concretizations
            Display concretizations

        --display-corpus
            Display corpus using uninstrumented fuzz target; to display with instrumentation, use
            --display-corpus-instrumented

        --display-crashes
            Display crashes

        --display-hangs
            Display hangs

        --display-impl-concretizations
            Display `impl` concretizations

        --display-queue
            Display work queue

        --exact
            Target name is an exact name rather than a substring

        --exit-code
            Exit with 0 if the time limit was reached, 1 for other programmatic aborts, and 2 if an
            error occurred; implies --no-ui, does not imply --run-until-crash or -- -V <SECONDS>

        --features <FEATURES>
            Space or comma separated list of features to activate

    -h, --help
            Print help information

        --list
            List fuzz targets

        --manifest-path <PATH>
            Path to Cargo.toml

        --no-default-features
            Do not activate the `default` feature

        --no-instrumentation
            Compile without instrumentation (for testing build process)

        --no-run
            Compile, but don't fuzz

        --no-ui
            Disable user interface

    -p, --package <PACKAGE>
            Package containing fuzz target

        --persistent
            Enable persistent mode fuzzing

        --pretty-print
            Pretty-print debug output when displaying/replaying

        --replay-corpus
            Replay corpus using uninstrumented fuzz target; to replay with instrumentation, use
            --replay-corpus-instrumented

        --replay-crashes
            Replay crashes

        --replay-hangs
            Replay hangs

        --replay-queue
            Replay work queue

        --reset
            Clear fuzzing data for one target, but leave corpus intact; to reset all targets, use
            --reset-all

        --resume
            Resume target's last fuzzing session

        --run-until-crash
            Stop fuzzing once a crash is found

        --target <TARGETNAME>
            DEPRECATED: Use just <TARGETNAME>

        --test <NAME>
            Integration test containing fuzz target

        --timeout <TIMEOUT>
            Number of milliseconds to consider a hang when fuzzing or replaying (equivalent to -- -t
            <TIMEOUT> when fuzzing)

    -V, --version
            Print version information
```

### Convenience macros

**Warning:** These macros are excluded from semantic versioning and may be removed in future versions of `test-fuzz`.

- **`dont_care!`**

  The `dont_care!` macro can be used to implement `serde::Serialize`/`serde::Deserialize` for types that are easy to construct and whose values you do not care to record. Intuitively, `dont_care!($ty, $expr)` says:

  - Skip values of type `$ty` when serializing.
  - Initialize values of type `$ty` with `$expr` when deserializing.

  More specifically, `dont_care!($ty, $expr)` expands to the following:

  ```rust
  impl serde::Serialize for $ty {
      fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
      where
          S: serde::Serializer,
      {
          ().serialize(serializer)
      }
  }

  impl<'de> serde::Deserialize<'de> for $ty {
      fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
      where
          D: serde::Deserializer<'de>,
      {
          <()>::deserialize(deserializer).map(|_| $expr)
      }
  }
  ```

  If `$ty` is a unit struct, then `$expr` can be be omitted. That is, `dont_care!($ty)` is equivalent to `dont_care!($ty, $ty)`.

- **`leak!`**

  The `leak!` macro can help to serialize target arguments that are references and whose types implement the [`ToOwned`](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html) trait. It is meant to be used with the [`convert`](#arguments) option.

  Specifically, an invocation of the following form declares a type `LeakedX`, and implements the `From` and `test_fuzz::Into` traits for it:

  ```rust
  leak!(X, LeakedX);
  ```

  One can then use `LeakedX` with the `convert` option as follows:

  ```rust
  #[test_fuzz::test_fuzz(convert = "&X, LeakedX")
  ```

  An example where `X` is [`Path`](https://doc.rust-lang.org/std/path/struct.Path.html) appears in [conversion.rs](examples/tests/conversion.rs#L5) in this repository.

  More generally, an invocation of the form `leak!($ty, $ident)` expands to the following:

  ```rust
  #[derive(Clone, std::fmt::Debug, serde::Deserialize, serde::Serialize)]
  struct $ident(<$ty as ToOwned>::Owned);

  impl From<&$ty> for $ident {
      fn from(ty: &$ty) -> Self {
          Self(ty.to_owned())
      }
  }

  impl test_fuzz::Into<&$ty> for $ident {
      fn into(self) -> &'static $ty {
          Box::leak(Box::new(self.0))
      }
  }
  ```

## `test-fuzz` package features

The features in this section apply to the `test-fuzz` package as a whole. Enable them in `test-fuzz`'s dependency specification as described in the [The Cargo Book](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features). For example, to enable the `auto_concretize` feature, use:

```toml
test-fuzz = { version = "2.0.2", features = ["auto_concretize"] }
```

The `test-fuzz` package currently supports the following features:

- **`auto_concretize`** - When this feature is enabled, `test-fuzz` tries to infer `impl` and non-`impl` concretizations. Success requires that a target be called with exactly one `impl` concretization and exactly one non-`impl` concretization during tests. Success is not guaranteed by these conditions, however.

  The implementation of `auto_concretize` uses the unstable language feature [`proc_macro_span`](https://github.com/rust-lang/rust/issues/54725). So enabling `auto_concretize` requires that targets be built with a nightly compiler.

- **Serde formats** - `test-fuzz` can serialize target arguments in multiple Serde formats. The following are the features used to select a format.

  - **`serde_bincode`** - [Bincode](https://github.com/bincode-org/bincode) (default)

  - **`serde_cbor`** - [Serde CBOR](https://github.com/pyfisch/cbor)

  - **`serde_cbor4ii`** - [CBOR 0x(4+4)9 0x49](https://github.com/quininer/cbor4ii)

## Auto-generated corpus files

`cargo-test-fuzz` can auto-generate values for types that implement certain traits. If all of a target's argument types implement such traits, `cargo-test-fuzz` can auto-generate corpus files for the target.

The traits that `cargo-test-fuzz` currently supports and the values generated for them are as follows:

| Trait(s)                          | Value(s)                                                           |
| --------------------------------- | ------------------------------------------------------------------ |
| `Bounded`                         | `T::min_value()`, `T::max_value()`                                 |
| `Bounded + Add + One`             | `T::min_value() + T::one()`                                        |
| `Bounded + Add + Div + Two`       | `T::min_value() / T::two() + T::max_value() / T::two()`            |
| `Bounded + Add + Div + Two + One` | `T::min_value() / T::two() + T::max_value() / T::two() + T::one()` |
| `Bounded + Sub + One`             | `T::max_value() - T::one()`                                        |
| `Default`                         | `T::default()`                                                     |

**Key**

- `Add` - [`core::ops::Add`](https://doc.rust-lang.org/beta/core/ops/trait.Add.html)
- `Bounded` - [`num_traits::bounds::Bounded`](https://docs.rs/num-traits/0.2.14/num_traits/bounds/trait.Bounded.html)
- `Default` - [`std::default::Default`](https://doc.rust-lang.org/std/default/trait.Default.html)
- `Div` - [`core::ops::Div`](https://doc.rust-lang.org/beta/core/ops/trait.Div.html)
- `One` - [`num_traits::One`](https://docs.rs/num-traits/0.2.14/num_traits/identities/trait.One.html)
- `Sub` - [`core::ops::Sub`](https://doc.rust-lang.org/beta/core/ops/trait.Sub.html)
- `Two` - `test_fuzz::runtime::traits::Two` (essentially `Add + One`)

## Environment variables

- **`TEST_FUZZ_LOG`** - During macro expansion:

  - If `TEST_FUZZ_LOG` is set to `1`, write all instrumented fuzz targets and module definitions to standard output.
  - If `TEST_FUZZ_LOG` is set to a crate name, write that crate's instrumented fuzz targets and module definitions to standard output.

  This can be useful for debugging.

- **`TEST_FUZZ_MANIFEST_PATH`** - When running a target from outside its package directory, find the package's `Cargo.toml` file at this location. One may need to set this environment variable when [`enable_in_production`](#arguments) is used.

- **`TEST_FUZZ_WRITE`** - Generate corpus files when not running tests for those targets for which [`enable_in_production`](#arguments) is set.

## Limitations

- **Clonable arguments** - A target's arguments must implement the [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) trait. The reason for this requirement is that the arguments are needed in two places: in a `test-fuzz`-internal function that writes corpus files, and in the body of the target function. To resolve this conflict, the arguments are cloned before being passed to the former.

- **Serializable / deserializable arguments** - In general, a target's arguments must implement the [`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html) and [`serde::Deserialize`](https://docs.serde.rs/serde/trait.Deserialize.html) traits, e.g., by [deriving them](https://serde.rs/derive.html). We say "in general" because `test-fuzz` knows how to handle certain special cases that wouldn't normally be serializable/deserializable. For example, an argument of type `&str` is converted to `String` when serializing, and back to a `&str` when deserializing. See also [`concretize` and `concretize_impl`](#arguments) above.

- **Global variables** - The fuzzing harnesses that `test-fuzz` implements do not initialize global variables. While [`execute_with`](#arguments) provides some remedy, it is not a complete solution. In general, fuzzing a function that relies on global variables requires ad-hoc methods.

- **[`convert`](#arguments) and [`concretize`](#arguments) / [`concretize_impl`](#arguments)** - These options are incompatible in the following sense. If a fuzz target's argument type is a type parameter, [`convert`](#arguments) will try to match the type parameter, not the type to which it is concretized. Supporting the latter would seem to require simulating type substitution as the compiler would perform it. However, this is not currently implemented.

## Tips and tricks

- `#[cfg(test)]` [is not enabled](https://github.com/rust-lang/rust/issues/45599#issuecomment-460488107) for integration tests. If your target is tested only by integration tests, then consider using [`enable_in_production`](#arguments) and [`TEST_FUZZ_WRITE`](#environment-variables) to generate a corpus. (Note the warning accompanying [`enable_in_production`](#arguments), however.)

- If you know the package in which your target resides, passing `-p <package>` to `cargo test`/[`cargo test-fuzz`](#cargo-test-fuzz-command) can significantly reduce build times. Similarly, if you know your target is called from only one integration test, passing `--test <name>` can reduce build times.

- Rust [won't allow you to](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types) implement `serde::Serialize` for other repositories' types. But you may be able to [patch](https://doc.rust-lang.org/edition-guide/rust-2018/cargo-and-crates-io/replacing-dependencies-with-patch.html) other repositories to make their types serializeble. Also, [`cargo-clone`](https://github.com/JanLikar/cargo-clone) can be useful for grabbing dependencies' repositories.

- [Serde attributes](https://serde.rs/attributes.html) can be helpful in implementing `serde::Serialize`/`serde::Deserialize` for difficult types.
