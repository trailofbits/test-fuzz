# Changelog

## 5.2.2

- Account for changes to `cargo afl --version` ([#422](https://github.com/trailofbits/test-fuzz/pull/422))

## 5.2.1

- Eliminate unnecessary dependence on `paste` ([#398](https://github.com/trailofbits/test-fuzz/pull/398))
- Update `itertools` to version `0.13` ([#400](https://github.com/trailofbits/test-fuzz/pull/400))
- Update `mio` to version `1.0` ([#406](https://github.com/trailofbits/test-fuzz/pull/406))
- Eliminate duplicate "AFL LLVM runtime was not built..." messages ([#415](https://github.com/trailofbits/test-fuzz/pull/415))

## 5.2.0

- Fix a bug causing incorrect exit codes to be produced ([3ab762f](https://github.com/trailofbits/test-fuzz/commit/3ab762f28ec73ae5692bce43267c539f56107545))
- FEATURE: `cargo-test-fuzz` now fuzzes all targets matching `TARGETNAME` concurrently, using at most all but one available cpu by default. If `TARGETNAME` is omitted, then `cargo-test-fuzz` fuzzes all targets concurrently. If there are not sufficiently many cpus to fuzz all targets simultaneously, then they are fuzzed in a time-sliced manner, in intervals of 20 minutes by default. ([c36d10d](https://github.com/trailofbits/test-fuzz/commit/c36d10d36fd81dde260c72fe02e7b52ba652f90b) and [8f36a0b](https://github.com/trailofbits/test-fuzz/commit/8f36a0b64b00b8615abb44baa3556fc66753a360))

## 5.1.0

- FEATURE: Add `cast_checks` feature ([#384](https://github.com/trailofbits/test-fuzz/pull/384))

## 5.0.0

- BREAKING CHANGE: Remove `auto_concretize` feature ([#336](https://github.com/trailofbits/test-fuzz/pull/336))
- FEATURE: Add `--max-total-time` option ([#323](https://github.com/trailofbits/test-fuzz/pull/323))
- FEATURE: Add `self_ty_in_mod_name` feature ([#328](https://github.com/trailofbits/test-fuzz/pull/328))
- Fix typo in `cargo-test-fuzz` help message ([#325](https://github.com/trailofbits/test-fuzz/pull/325))
- Deprecate `concretizations` terminology in favor of `generic-args` ([#340](https://github.com/trailofbits/test-fuzz/pull/340))
- Give correct advice for installing `cargo-afl` when it cannot be found ([9101dbe](https://github.com/trailofbits/test-fuzz/commit/9101dbee8e45d5c7aeb77a3b94f316e2b9aa16bd))
- Properly handle receiverless trait functions ([#346](https://github.com/trailofbits/test-fuzz/pull/346))

## 4.0.5

- Format macro-generated code with `prettyplease` ([#314](https://github.com/trailofbits/test-fuzz/pull/314))
- Update `afl` to version 0.15.0 ([#321](https://github.com/trailofbits/test-fuzz/pull/321))

## 4.0.4

- Add `auto_concretize` deprecation message ([#305](https://github.com/trailofbits/test-fuzz/pull/305))

## 4.0.3

- Fix install command in documentation ([#294](https://github.com/trailofbits/test-fuzz/pull/294))&mdash;thanks [@maxammann](https://github.com/maxammann)
- Other documentation improvements ([#295](https://github.com/trailofbits/test-fuzz/pull/295), [#297](https://github.com/trailofbits/test-fuzz/pull/297), [#298](https://github.com/trailofbits/test-fuzz/pull/298), [#300](https://github.com/trailofbits/test-fuzz/pull/300))
- Add unstable utility functions `serialize_ref_mut` and `deserialize_ref_mut` ([#301](https://github.com/trailofbits/test-fuzz/pull/301))

## 4.0.2

- Significant refactoring regarding Serde format handling and Cargo feature use, but users should experience no differences ([#284](https://github.com/trailofbits/test-fuzz/pull/284))
- Correct error message when `cargo-afl` cannot be found ([#286](https://github.com/trailofbits/test-fuzz/pull/286))

## 4.0.1

- Remove last reference to `syn` 1.0 ([#244](https://github.com/trailofbits/test-fuzz/pull/244))

## 4.0.0

- BREAKING CHANGE: The `--timeout` option now uses seconds instead of milliseconds ([#219](https://github.com/trailofbits/test-fuzz/pull/219))&mdash;thanks [@dhruvdabhi101](https://github.com/dhruvdabhi101)
- BREAKING CHANGE: The following deprecated options/functions have been removed ([#234](https://github.com/trailofbits/test-fuzz/pull/234)):
  - Options `--display-<OBJECT>` (use `--display <OBJECT>` with no hyphen)
  - Options `--replay-<OBJECT>` (use `--replay <OBJECT>` with no hyphen)
  - Option `--target <TARGETNAME>` (use just `<TARGETNAME>`)
  - Function `cargo_test_fuzz`
- BREAKING CHANGE: `test-fuzz` is now licensed and distributed under the AGPLv3 license with the [Macros and Inline Functions Exception](https://spdx.org/licenses/mif-exception.html). See the [README](https://github.com/trailofbits/test-fuzz#license) for additional details. ([#241](https://github.com/trailofbits/test-fuzz/pull/241))
- Fix a bug involving mutable slices and `str`s ([#230](https://github.com/trailofbits/test-fuzz/pull/230))&mdash;thanks [@0xalpharush](https://github.com/0xalpharush) for reporting the bug

## 3.1.0

- Update dependencies, including `afl` to version 0.13.0 ([c1707f5](https://github.com/trailofbits/test-fuzz/commit/c1707f5c09e9c68113699c67be13fa4944a94405))

## 3.0.5

- Update help message ([#153](https://github.com/trailofbits/test-fuzz/pull/153))

## 3.0.4

- Work around "pizza mode" bug ([#137](https://github.com/trailofbits/test-fuzz/pull/137))

## 3.0.3

- Don't assume converted arguments are cloneable ([#130](https://github.com/trailofbits/test-fuzz/pull/130))
- Handle mutable arguments ([#132](https://github.com/trailofbits/test-fuzz/pull/132))

## 3.0.2

- Simplify command line interface ([#120](https://github.com/trailofbits/test-fuzz/pull/120))

## 3.0.1

- Handle unused lifetime parameters ([#116](https://github.com/trailofbits/test-fuzz/pull/116))

## 3.0.0

- BREAKING CHANGE: Make `afl` an optional dependency enabled by `--persistent`. This is a breaking change in the following sense. If one tries to use `cargo-test-fuzz` 2.0.x with a target compiled with the new version of `test-fuzz`, one will receive a `` ... does not depend on `afl` `` error. ([#114](https://github.com/trailofbits/test-fuzz/pull/114))

## 2.0.5

- Improvements and bug fixes related to the `convert` option ([#107](https://github.com/trailofbits/test-fuzz/pull/107), [#109](https://github.com/trailofbits/test-fuzz/pull/109), and [#111](https://github.com/trailofbits/test-fuzz/pull/111))
- Add `--verbose` option ([#108](https://github.com/trailofbits/test-fuzz/pull/108))

## 2.0.4

- Eliminate use of `std::array::IntoIter::new` ([#106](https://github.com/trailofbits/test-fuzz/pull/106))

## 2.0.3

- Handle structs with lifetime parameters ([#103](https://github.com/trailofbits/test-fuzz/pull/103))

## 2.0.2

- Update afl requirement from =0.11.1 to =0.12.0 ([77a837d](https://github.com/trailofbits/test-fuzz/commit/77a837db9f92f921b58ad63fa3ee099c5e1d95ff))

## 2.0.1

- It is no longer necessary to specify `default-feature = false` when selecting a Serde format. ([#81](https://github.com/trailofbits/test-fuzz/pull/81) and [#85](https://github.com/trailofbits/test-fuzz/pull/85))
- Add [`cbor4ii`](https://github.com/quininer/cbor4ii) as a Serde format ([0518100](https://github.com/trailofbits/test-fuzz/commit/05181001f6bd9ee235174c1d63d24d9e7475e4b1))

## 2.0.0

- Add `leak!` convenience macro ([cc74b10](https://github.com/trailofbits/test-fuzz/commit/cc74b10a645819dc3b983c03cc6a6f4c05a952a5))
- Properly handle case of uninstalled `cargo-afl` ([436bc6e](https://github.com/trailofbits/test-fuzz/commit/436bc6e9e7b32ce1ffb3a05bce766b83b9ade329))
- DEPRECATED: `--target` is no longer needed to name targets ([117580a](https://github.com/trailofbits/test-fuzz/commit/117580a19b70b7c0fd4e05e7ef8ffd4b7d4fe7b8))
- BREAKING CHANGE: Retire builtin serialization/deserialization support for `Arc` in favor of `serde`'s ([31c41b2](https://github.com/trailofbits/test-fuzz/commit/31c41b249d266e9327e48353454289ce49d80e30))

## 1.0.4

- Account for features and manifest path when obtaining Cargo metadata ([#64](https://github.com/trailofbits/test-fuzz/pull/64))

## 1.0.3

- Add `--exit-code` option ([8bcbc2f](https://github.com/trailofbits/test-fuzz/commit/8bcbc2ff539d553984dc5a645a65213e9b4f3adc))

## 1.0.2

- Support lifetime arguments ([beae251](https://github.com/trailofbits/test-fuzz/commit/beae251a4b3d915f0d7fae7a4a1ce9fb091baa0d))
- Allow conversion of types beyond path types ([6a1595f](https://github.com/trailofbits/test-fuzz/commit/6a1595f201ca52ee81a56a9567476363d0c69fec))
- Fully qualify Result in dont_care macro ([fe598af](https://github.com/trailofbits/test-fuzz/commit/fe598af32768a9ceecb8090fb1dd1efcef91a6fb))

## 1.0.1

- Bump AFL version ([#54](https://github.com/trailofbits/test-fuzz/pull/54))
- Improve build times ([#55](https://github.com/trailofbits/test-fuzz/pull/55))

## 1.0.0

- Add `auto_concretize` feature ([408e4c2](https://github.com/trailofbits/test-fuzz/commit/408e4c2bb7049095dea2e05a2a7fe76ba0931a6d))
- Add `convert` ([605f050](https://github.com/trailofbits/test-fuzz/commit/605f0505fc752a18b3a546a75676e6b2bc1d08c6)) and `execute_with` ([5742988](https://github.com/trailofbits/test-fuzz/commit/5742988f8e9b7718f425de9745bf758a22d2d0ea)) `test-fuzz` macro options
- Add `--manifest-path` ([5a45236](https://github.com/trailofbits/test-fuzz/commit/5a452369960265a05b01f005bee5bd6c012c4c5b)) and `--test` ([d080686](https://github.com/trailofbits/test-fuzz/commit/d0806868cecc6cdd43d681cf84d9f7b50dc64771)) `cargo-test-fuzz` options
- Allow `--features` to be passed more than once ([15d89ae](https://github.com/trailofbits/test-fuzz/commit/15d89ae7024e4c8a0011352c3079b07d05e42606))
- Better error reporting ([6ceff54](https://github.com/trailofbits/test-fuzz/commit/6ceff54d48202f640010691afaff9ce3460c91c8))
- BREAKING CHANGE: `TEST_FUZZ_LOG` can be set to a crate name ([a9b9d21](https://github.com/trailofbits/test-fuzz/commit/a9b9d21b685c030b8f9aa535f0d7886cec092ef3))
- BREAKING CHANGE: `no_auto` is now `no_auto_generate` ([100fae4](https://github.com/trailofbits/test-fuzz/commit/100fae47feb4c3c609a9b9b96ea7c696a7a45b97))

## 0.1.0

- Initial release
