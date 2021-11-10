# Changelog

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
