# Libfuzzer integration

## `libtest` calls `libfuzzer`: why?

`libfuzzer` and `libtest` both want to provide a `main` function. Which one should we use?

In each case, we get a kind of "callback" from the `main` function. This gives us an opportunity to call the other `main` function. So the question becomes: should `libfuzzer` call `libtest`, or should `libtest` call `libfuzzer`? I.e.,

```
libfuzzer --> libtest
```

or

```
libtest --> libfuzzer
```

We have chosen the latter (i.e., `libtest` calls `libfuzzer`) because doing the former (i.e., `libfuzzer` calls `libtest`) would be difficult:

1. `test-fuzz` relies heavily on `libtest`'s command line option parsing. If `libfuzzer` called `libtest`, we would need to find an alternative way to pass options to `libtest`. Moreover, this alternative way would ony apply when `libfuzzer` was selected as the fuzzer, so it would need to be turned on and off. In short: it would be a hassle.
2. The code that `libfuzzer`'s `main` function calls is called repeatedly. If `libfuzzer`'s main function called `libtest`'s main function, the calls would happen repeatedly.

## Implementation

### `LIBFUZZER_FUZZ_TARGET`

`libfuzzer` expects there to be just one fuzz target (`rust_fuzzer_test_input`) which `libfuzzer` calls from its `main` function. But `test-fuzz` allows a binary to contain multiple fuzz targets. So we need a way to tell `libfuzzer` which one to call.

Our solution is to use a global atomic pointer, `LIBFUZZER_FUZZ_TARGET`. This pointer is set from within the macro generated code after `libtest` is called, but before control is handed to `libfuzzer`. Our `rust_fuzzer_test_input` fuzz target reads that pointer and (unsafely) calls the pointed-to function.
