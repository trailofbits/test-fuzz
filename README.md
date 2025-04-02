# test-fuzz

`test-fuzz` is a Cargo subcommand and a collection of Rust macros to automate certain tasks related to fuzzing with [`afl.rs`], including:

- generating a fuzzing corpus
- implementing a fuzzing harness

`test-fuzz` accomplishes these (in part) using Rust's testing facilities. For example, to generate a fuzzing corpus, `test-fuzz` records a target's arguments each time it is called during an invocation of `cargo test`. Similarly, `test-fuzz` implements a fuzzing harness as an additional test in a `cargo-test`-generated binary. This tight integration with Rust's testing facilities is what motivates the name **`test`**`-fuzz`.

**Contents**

1. [Installation]
2. [Overview]
3. [Components]
   - [`test_fuzz` macro]
   - [`test_fuzz_impl` macro]
   - [`cargo test-fuzz` command]
   - [Convenience functions and macros]
4. [`test-fuzz` package features]
5. [Auto-generated corpus files]
6. [Environment variables]
7. [Limitations]
8. [Tips and tricks]
9. [Semantic versioning policy]
10. [License]

## Installation

Install `cargo-test-fuzz` and [`afl.rs`] with the following command:

```sh
cargo install cargo-test-fuzz cargo-afl
```

## Overview

Fuzzing with `test-fuzz` is essentially three steps:\*

1. **Identify a fuzz target**:
   - Add the following `dependencies` to the target crate's `Cargo.toml` file:
     ```toml
     serde = "*"
     test-fuzz = "*"
     ```
   - Precede the target function with the [`test_fuzz`] macro:
     ```rust
     #[test_fuzz::test_fuzz]
     fn foo(...) {
         ...
     }
     ```
2. **Generate a corpus** by running `cargo test`:
   ```
   cargo test
   ```
3. **Fuzz your target** by running [`cargo test-fuzz`]:
   ```
   cargo test-fuzz foo
   ```

\* An additional, preliminary step may be necessary following a reboot:

```sh
cargo afl system-config
```

Note that the above command runs `sudo` internally. Hence, you may be prompted to enter your password.

## Components

### `test_fuzz` macro

Preceding a function with the `test_fuzz` macro indicates that the function is a fuzz target.

The primary effects of the `test_fuzz` macro are:

- Add instrumentation to the target to serialize its arguments and write them to a corpus file each time the target is called. The instrumentation is guarded by `#[cfg(test)]` so that corpus files are generated only when running tests (however, see [`enable_in_production`] below).
- Add a test to read and deserialize arguments from standard input and apply the target to them. The test checks an environment variable, set by [`cargo test-fuzz`], so that the test does not block trying to read from standard input during a normal invocation of `cargo test`. The test is enclosed in a module to reduce the likelihood of a name collision. Currently, the name of the module is `target_fuzz`, where `target` is the name of the target (however, see [`rename`] below).

#### Arguments

##### `bounds = "where_predicates"`

Impose `where_predicates` (e.g., trait bounds) on the struct used to serialize/deserialize arguments. This may be necessary, e.g., if a target's argument type is an associated type. For an example, see [associated_type.rs] in this repository.

##### `generic_args = "parameters"`

Use `parameters` as the target's type parameters when fuzzing. Example:

```rust
#[test_fuzz(generic_args = "String")]
fn foo<T: Clone + Debug + Serialize>(x: &T) {
    ...
}
```

Note: The target's arguments must be serializable for **every** instantiation of its type parameters. But the target's arguments are required to be deserializable only when the target is instantiated with `parameters`.

##### `impl_generic_args = "parameters"`

Use `parameters` as the target's `Self` type parameters when fuzzing. Example:

```rust
#[test_fuzz_impl]
impl<T: Clone + Debug + Serialize> for Foo {
    #[test_fuzz(impl_generic_args = "String")]
    fn bar(&self, x: &T) {
        ...
    }
}
```

Note: The target's arguments must be serializable for **every** instantiation of its `Self` type parameters. But the target's arguments are required to be deserializable only when the target's `Self` is instantiated with `parameters`.

##### `convert = "X, Y"`

When serializing the target's arguments, convert values of type `X` to type `Y` using `Y`'s implementation of `From<X>`, or of type `&X` to type `Y` using `Y`'s implementation of the non-standard trait `test_fuzz::FromRef<X>`. When deserializing, convert those values back to type `X` using `Y`'s implementation of the non-standard trait `test_fuzz::Into<X>`.

That is, use of `convert = "X, Y"` must be accompanied by certain implementations. If `X` implements [`Clone`], then `Y` may implement the following:

```rust
impl From<X> for Y {
    fn from(x: X) -> Self {
        ...
    }
}
```

If `X` does not implement [`Clone`], then `Y` must implement the following:

```rust
impl test_fuzz::FromRef<X> for Y {
    fn from_ref(x: &X) -> Self {
        ...
    }
}
```

Additionally, `Y` must implement the following (regardless of whether `X` implements [`Clone`]):

```rust
impl test_fuzz::Into<X> for Y {
    fn into(self) -> X {
        ...
    }
}
```

The definition of `test_fuzz::Into` is identical to that of [`std::convert::Into`]. The reason for using a non-standard trait is to avoid conflicts that could arise from blanket implementations of standard traits.

##### `enable_in_production`

Generate corpus files when not running tests, provided the environment variable [`TEST_FUZZ_WRITE`] is set. The default is to generate corpus files only when running tests, regardless of whether [`TEST_FUZZ_WRITE`] is set. When running a target from outside its package directory, set [`TEST_FUZZ_MANIFEST_PATH`] to the path of the package's `Cargo.toml` file.

**WARNING**: Setting `enable_in_production` could introduce a denial-of-service vector. For example, setting this option for a function that is called many times with different arguments could fill up the disk. The check of [`TEST_FUZZ_WRITE`] is meant to provide some defense against this possibility. Nonetheless, consider this option carefully before using it.

##### `execute_with = "function"`

Rather than call the target directly:

- construct a closure of type `FnOnce() -> R`, where `R` is the target's return type, so that calling the closure calls the target;
- call `function` with the closure.

Calling the target in this way allows `function` to set up the call's environment. This can be useful, e.g., for fuzzing [Substrate externalities].

##### `no_auto_generate`

Do not try to [auto-generate corpus files] for the target.

##### `only_generic_args`

Record the target's generic args when running tests, but do not generate corpus files and do not implement a fuzzing harness. This can be useful when the target is a generic function, but it is unclear what type parameters should be used for fuzzing.

The intended workflow is: enable `only_generic_args`, then run `cargo test` followed by `cargo test-fuzz --display generic-args`. One of the resulting generic args might be usable as `generic_args`'s `parameters`. Similarly, generic args resulting from `cargo test-fuzz --display impl-generic-args` might be usable as `impl_generic_args`'s `parameters`.

Note, however, that just because a target was called with certain parameters during tests, it does not imply the target's arguments are serializable/deserializable when those parameters are used. The results of `--display generic-args`/`--display impl-generic-args` are merely suggestive.

##### `rename = "name"`

Treat the target as though its name is `name` when adding a module to the enclosing scope. Expansion of the `test_fuzz` macro adds a module definition to the enclosing scope. By default, the module is named as follows:

- If the target does not appear in an `impl` block, the module is named `target_fuzz__`, where `target` is the name of the target.
- If the target appears in an `impl` block, the module is named `path_target_fuzz__`, where `path` is the path of the `impl`'s `Self` type converted to snake case and joined with `_`.

However, use of this option causes the module to instead be named `name_fuzz__`. Example:

```rust
#[test_fuzz(rename = "bar")]
fn foo() {}

// Without the use of `rename`, a name collision and compile error would result.
mod foo_fuzz__ {}
```

#### Serde field attributes on function arguments

The `test_fuzz` macro allows [Serde field attributes] to be applied to function arguments. This provides another tool for dealing with difficult types.

The following is an example. Traits `serde::Serialize` and `serde::Deserialize` cannot be derived for `Context` because it contains a `Mutex`. However, `Context` implements `Default`. So applying `#[serde(skip)]` to the `Context` argument causes it to be skipped when serializing, and to take its default value when deserializing.

```rust
use std::sync::Mutex;

// Traits `serde::Serialize` and `serde::Deserialize` cannot be derived for `Context` because it
// contains a `Mutex`.
#[derive(Default)]
struct Context {
    lock: Mutex<()>,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }
}

#[test_fuzz::test_fuzz]
fn target(#[serde(skip)] context: Context, x: i32) {
    assert!(x >= 0);
}
```

Note that when Serde field attributes are applied to an argument, the `test_fuzz` macro performs no other [conversions] on the argument.

### `test_fuzz_impl` macro

Whenever the [`test_fuzz`] macro is used in an `impl` block,
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

The reason for this requirement is as follows. Expansion of the [`test_fuzz`] macro adds a module definition to the enclosing scope. However, a module definition cannot appear inside an `impl` block. Preceding the `impl` with the `test_fuzz_impl` macro causes the module to be added outside the `impl` block.

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
   cargo test-fuzz foo --display corpus
   ```
3. Fuzz target `foo`
   ```
   cargo test-fuzz foo
   ```
4. Replay crashes found for target `foo`
   ```
   cargo test-fuzz foo --replay crashes
   ```

#### Usage

```
Usage: cargo test-fuzz [OPTIONS] [TARGETNAME] [-- <ARGS>...]

Arguments:
  [TARGETNAME]  String that fuzz target's name must contain
  [ARGS]...     Arguments for the fuzzer

Options:
      --backtrace                 Display backtraces
      --consolidate               Move one target's crashes, hangs, and work queue to its corpus; to
                                  consolidate all targets, use --consolidate-all
      --cpus <N>                  Fuzz using at most <N> cpus; default is all but one
      --display <OBJECT>          Display corpus, crashes, generic args, `impl` generic args, hangs,
                                  or work queue. By default, an uninstrumented fuzz target is used.
                                  To display with instrumentation, append `-instrumented` to
                                  <OBJECT>, e.g., --display corpus-instrumented.
      --exact                     Target name is an exact name rather than a substring
      --exit-code                 Exit with 0 if the time limit was reached, 1 for other
                                  programmatic aborts, and 2 if an error occurred; implies --no-ui,
                                  does not imply --run-until-crash or --max-total-time <SECONDS>
      --features <FEATURES>       Space or comma separated list of features to activate
      --list                      List fuzz targets
      --manifest-path <PATH>      Path to Cargo.toml
      --max-total-time <SECONDS>  Fuzz at most <SECONDS> of time (equivalent to -- -V <SECONDS>)
      --no-default-features       Do not activate the `default` feature
      --no-run                    Compile, but don't fuzz
      --no-ui                     Disable user interface
  -p, --package <PACKAGE>         Package containing fuzz target
      --persistent                Enable persistent mode fuzzing
      --pretty                    Pretty-print debug output when displaying/replaying
      --release                   Build in release mode
      --replay <OBJECT>           Replay corpus, crashes, hangs, or work queue. By default, an
                                  uninstrumented fuzz target is used. To replay with instrumentation
                                  append `-instrumented` to <OBJECT>, e.g., --replay
                                  corpus-instrumented.
      --reset                     Clear fuzzing data for one target, but leave corpus intact; to
                                  reset all targets, use --reset-all
      --resume                    Resume target's last fuzzing session
      --run-until-crash           Stop fuzzing once a crash is found
      --slice <SECONDS>           If there are not sufficiently many cpus to fuzz all targets
                                  simultaneously, fuzz them in intervals of <SECONDS> [default:
                                  1200]
      --test <NAME>               Integration test containing fuzz target
      --timeout <TIMEOUT>         Number of seconds to consider a hang when fuzzing or replaying
                                  (equivalent to -- -t <TIMEOUT * 1000> when fuzzing)
      --verbose                   Show build output when displaying/replaying
  -h, --help                      Print help
  -V, --version                   Print version

Try `cargo afl fuzz --help` to see additional fuzzer options.
```

When using the `--display` option, any output written to stderr by the target is shown. This includes output from `eprintln!` statements, as well as from debugging macros like `dbg!`. This can be useful for understanding what's happening in your code when processing specific inputs.

The `--display` and `--replay` options can be passed together, allowing you to both view and replay corpus entries in a single command, for example:

```
cargo test-fuzz foo --display corpus --replay corpus
```

### Convenience functions and macros

**Warning:** These utilties are excluded from semantic versioning and may be removed in future versions of `test-fuzz`.

#### `dont_care!`

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

If `$ty` is a unit struct, then `$expr` can be omitted. That is, `dont_care!($ty)` is equivalent to `dont_care!($ty, $ty)`.

#### `leak!`

The `leak!` macro can help to serialize target arguments that are references and whose types implement the [`ToOwned`] trait. It is meant to be used with the [`convert`] option.

Specifically, an invocation of the following form declares a type `LeakedX`, and implements the `From` and `test_fuzz::Into` traits for it:

```rust
leak!(X, LeakedX);
```

One can then use `LeakedX` with the `convert` option as follows:

```rust
#[test_fuzz::test_fuzz(convert = "&X, LeakedX")
```

An example where `X` is [`Path`] appears in [conversion.rs] in this repository.

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

#### `serialize_ref` / `deserialize_ref`

`serialize_ref` and `deserialize_ref` function similar to `leak!`, but they are meant to be used wth Serde's [`serialize_with`] and [`deserialize_with`] field attributes (respectively).

```rust
fn serialize_ref<S, T>(x: &&T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    <T as serde::Serialize>::serialize(*x, serializer)
}

fn deserialize_ref<'de, D, T>(deserializer: D) -> Result<&'static T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    let x = <T as serde::de::Deserialize>::deserialize(deserializer)?;
    Ok(Box::leak(Box::new(x)))
}
```

#### `serialize_ref_mut` / `deserialize_ref_mut`

`serialize_ref_mut` and `deserialize_ref_mut` are similar to `serialize_ref` and `deserialize_ref` (respectively), expect they operate on mutable references instead of immutable ones.

## `test-fuzz` package features

The features in this section apply to the `test-fuzz` package as a whole. Enable them in `test-fuzz`'s dependency specification as described in the [The Cargo Book]. For example, to enable the `cast_checks` feature, use:

```toml
test-fuzz = { version = "*", features = ["cast_checks"] }
```

The `test-fuzz` package currently supports the following features:

### `cast_checks`

Use [`cast_checks`] to automatically check target functions for invalid casts.

Note that this feature enables `cast_checks` only for functions annotated with the [`test_fuzz` macro], not for the functions they call.

### Serde formats

`test-fuzz` can serialize target arguments in multiple Serde formats. The following are the features used to select a format.

- `serde_bincode` - [Bincode] (default)

- `serde_postcard` - [Postcard]

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

- `Add` - [`core::ops::Add`]
- `Bounded` - [`num_traits::bounds::Bounded`]
- `Default` - [`std::default::Default`]
- `Div` - [`core::ops::Div`]
- `One` - [`num_traits::One`]
- `Sub` - [`core::ops::Sub`]
- `Two` - `test_fuzz::runtime::traits::Two` (essentially `Add + One`)

## Environment variables

### `TEST_FUZZ_LOG`

During macro expansion:

- If `TEST_FUZZ_LOG` is set to `1`, write all instrumented fuzz targets and module definitions to standard output.
- If `TEST_FUZZ_LOG` is set to a crate name, write that crate's instrumented fuzz targets and module definitions to standard output.

This can be useful for debugging.

### `TEST_FUZZ_MANIFEST_PATH`

When running a target from outside its package directory, find the package's `Cargo.toml` file at this location. One may need to set this environment variable when [`enable_in_production`] is used.

### `TEST_FUZZ_WRITE`

Generate corpus files when not running tests for those targets for which [`enable_in_production`] is set.

## Limitations

### Clonable arguments

A target's arguments must implement the [`Clone`] trait. The reason for this requirement is that the arguments are needed in two places: in a `test-fuzz`-internal function that writes corpus files, and in the body of the target function. To resolve this conflict, the arguments are cloned before being passed to the former.

### Serializable / deserializable arguments

In general, a target's arguments must implement the [`serde::Serialize`] and [`serde::Deserialize`] traits, e.g., by [deriving them]. We say "in general" because `test-fuzz` knows how to handle certain special cases that wouldn't normally be serializable/deserializable. For example, an argument of type `&str` is converted to `String` when serializing, and back to a `&str` when deserializing. See also [`generic_args`] and [`impl_generic_args`] above.

### Global variables

The fuzzing harnesses that `test-fuzz` implements do not initialize global variables. While [`execute_with`] provides some remedy, it is not a complete solution. In general, fuzzing a function that relies on global variables requires ad-hoc methods.

### [`convert`] and [`generic_args`] / [`impl_generic_args`]

These options are incompatible in the following sense. If a fuzz target's argument type is a type parameter, [`convert`] will try to match the type parameter, not the type to which the parameter is set. Supporting the latter would seem to require simulating type substitution as the compiler would perform it. However, this is not currently implemented.

## Tips and tricks

- `#[cfg(test)]` [is not enabled] for integration tests. If your target is tested only by integration tests, then consider using [`enable_in_production`] and [`TEST_FUZZ_WRITE`] to generate a corpus. (Note the warning accompanying [`enable_in_production`], however.)

- If you know the package in which your target resides, passing `-p <package>` to `cargo test`/[`cargo test-fuzz`] can significantly reduce build times. Similarly, if you know your target is called from only one integration test, passing `--test <name>` can reduce build times.

- Rust [won't allow you to] implement `serde::Serialize` for other repositories' types. But you may be able to [patch] other repositories to make their types serializable. Also, [`cargo-clone`] can be useful for grabbing dependencies' repositories.

- [Serde attributes] can be helpful in implementing `serde::Serialize`/`serde::Deserialize` for difficult types.

## Semantic versioning policy

We reserve the right to change the format of corpora, crashes, hangs, and work queues, and to consider such changes non-breaking.

## License

`test-fuzz` is licensed and distributed under the AGPLv3 license with the [Macros and Inline Functions Exception]. In plain language, using the [`test_fuzz` macro], the [`test_fuzz_impl` macro], or `test-fuzz`'s [convenience functions and macros] in your software does not require it to be covered by the AGPLv3 license.

[Auto-generated corpus files]: #auto-generated-corpus-files
[Bincode]: https://github.com/bincode-org/bincode
[Components]: #components
[Convenience functions and macros]: #convenience-functions-and-macros
[Environment variables]: #environment-variables
[Installation]: #installation
[License]: #license
[Limitations]: #limitations
[Macros and Inline Functions Exception]: https://spdx.org/licenses/mif-exception.html
[Overview]: #overview
[Postcard]: https://github.com/jamesmunns/postcard
[Serde attributes]: https://serde.rs/attributes.html
[Serde field attributes]: https://serde.rs/field-attrs.html
[Substrate externalities]: https://substrate.dev/docs/en/knowledgebase/runtime/tests#mock-runtime-storage
[The Cargo Book]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features
[Tips and tricks]: #tips-and-tricks
[`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
[`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
[`TEST_FUZZ_MANIFEST_PATH`]: #test_fuzz_manifest_path
[`TEST_FUZZ_WRITE`]: #test_fuzz_write
[`ToOwned`]: https://doc.rust-lang.org/std/borrow/trait.ToOwned.html
[`afl.rs`]: https://github.com/rust-fuzz/afl.rs
[`cargo test-fuzz` command]: #cargo-test-fuzz-command
[`cargo test-fuzz`]: #cargo-test-fuzz-command
[`cargo-clone`]: https://github.com/JanLikar/cargo-clone
[`cast_checks`]: https://github.com/trailofbits/cast_checks
[`convert`]: #convert--x-y
[`core::ops::Add`]: https://doc.rust-lang.org/beta/core/ops/trait.Add.html
[`core::ops::Div`]: https://doc.rust-lang.org/beta/core/ops/trait.Div.html
[`core::ops::Sub`]: https://doc.rust-lang.org/beta/core/ops/trait.Sub.html
[`deserialize_with`]: https://serde.rs/field-attrs.html#deserialize_with
[`enable_in_production`]: #enable_in_production
[`execute_with`]: #execute_with--function
[`generic_args`]: #generic_args--parameters
[`impl_generic_args`]: #impl_generic_args--parameters
[`num_traits::One`]: https://docs.rs/num-traits/0.2.14/num_traits/identities/trait.One.html
[`num_traits::bounds::Bounded`]: https://docs.rs/num-traits/0.2.14/num_traits/bounds/trait.Bounded.html
[`proc_macro_span`]: https://github.com/rust-lang/rust/issues/54725
[`rename`]: #rename--name
[`serde::Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
[`serde::Serialize`]: https://docs.serde.rs/serde/trait.Serialize.html
[`serialize_with`]: https://serde.rs/field-attrs.html#serialize_with
[`std::convert::Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
[`std::default::Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
[`test-fuzz` package features]: #test-fuzz-package-features
[`test_fuzz_impl` macro]: #test_fuzz_impl-macro
[`test_fuzz` macro]: #test_fuzz-macro
[`test_fuzz`]: #test_fuzz-macro
[associated_type.rs]: https://github.com/trailofbits/test-fuzz/blob/master/examples/tests/associated_type.rs#L26
[auto-generate corpus files]: #auto-generated-corpus-files
[conversion.rs]: https://github.com/trailofbits/test-fuzz/blob/master/examples/tests/conversion.rs#L5
[conversions]: #serializable--deserializable-arguments
[deriving them]: https://serde.rs/derive.html
[is not enabled]: https://github.com/rust-lang/rust/issues/45599#issuecomment-460488107
[patch]: https://doc.rust-lang.org/edition-guide/rust-2018/cargo-and-crates-io/replacing-dependencies-with-patch.html
[won't allow you to]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types
