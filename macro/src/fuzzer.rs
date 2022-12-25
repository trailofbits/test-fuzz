use super::Components;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// smoelius: Do not set the panic hook when replaying. Leave cargo test's
// panic hook in place.
#[cfg(any(fuzzer_default, feature = "__fuzzer_aflplusplus"))]
pub(crate) fn args_entry_stmts(components: &Components) -> TokenStream2 {
    let Components {
        set_panic_hook,
        take_panic_hook,
        input_args,
        output_args,
        args_ret_ty,
        output_ret,
        call_in_environment,
        ..
    } = components;

    let call_in_environment_with_deserialized_arguments = quote! {
        let ret: Option< #args_ret_ty > = args.map(|mut args|
            #call_in_environment
        );
    };

    quote! {
        if test_fuzz::runtime::display_enabled()
            || test_fuzz::runtime::replay_enabled()
        {
            #input_args
            if test_fuzz::runtime::display_enabled() {
                #output_args
            }
            if test_fuzz::runtime::replay_enabled() {
                #call_in_environment_with_deserialized_arguments
                #output_ret
            }
        } else {
            #set_panic_hook
            #input_args
            #call_in_environment_with_deserialized_arguments
            #take_panic_hook
        }
    }
}

#[cfg(feature = "__fuzzer_aflplusplus_persistent")]
pub(crate) fn args_entry_stmts(components: &Components) -> TokenStream2 {
    let Components {
        combined_concretization,
        set_panic_hook,
        take_panic_hook,
        args_ret_ty,
        call_in_environment,
        ..
    } = components;

    quote! {
        if test_fuzz::runtime::display_enabled()
            || test_fuzz::runtime::replay_enabled()
        {
            panic!("Displaying/replaying with `aflplusplus-persistent` is not supported. Try \
            passing --no-instrumentation.");
        } else {
            #set_panic_hook

            test_fuzz::afl::fuzz!(|data: &[u8]| {
                let mut args = UsingReader::<_>::read_args #combined_concretization (data);
                let ret: Option< #args_ret_ty > = args.map(|mut args|
                    #call_in_environment
                );
            });

            #take_panic_hook
        }
    }
}

#[cfg(feature = "__fuzzer_libfuzzer")]
pub(crate) fn args_entry_stmts(components: &Components) -> TokenStream2 {
    let Components {
        combined_concretization,
        output_args,
        args_ret_ty,
        output_ret,
        call_in_environment,
        ..
    } = components;

    quote! {
        static RET: std::sync::Mutex<Option< #args_ret_ty >> = std::sync::Mutex::new(None);

        fn libfuzzer_fuzz_target(data: &[u8]) {
            let mut args = UsingReader::<_>::read_args #combined_concretization (data);
            if let Some(ret) = args.map(|mut args|
                #call_in_environment
            ) {
                let mut lock = RET.lock().unwrap();
                *lock = Some(ret);
            }
        }

        let libfuzzer_fuzz_target_ptr = libfuzzer_fuzz_target as fn(&[u8]);
        test_fuzz::runtime::libfuzzer::LIBFUZZER_FUZZ_TARGET.store(
            unsafe { std::mem::transmute::<_, *mut ()>(libfuzzer_fuzz_target_ptr) },
            std::sync::atomic::Ordering::SeqCst
        );

        let mut code = 0;

        if test_fuzz::runtime::display_enabled()
            || test_fuzz::runtime::replay_enabled()
        {
            let data = {
                use std::io::Read;
                let mut data = Vec::new();
                std::io::stdin().read_to_end(&mut data).expect("Could not read from `stdin`");
                data
            };

            if test_fuzz::runtime::display_enabled() {
                let args = UsingReader::<_>::read_args #combined_concretization (data.as_slice());
                #output_args
            }

            // smoelius: If replaying with instrumentation, call `rust_fuzzer_test_input` directly.
            if test_fuzz::runtime::replay_enabled() {
                extern "C" {
                    #[allow(improper_ctypes)]
                    fn rust_fuzzer_test_input(input: &[u8]) -> i32;
                }
                code = unsafe { rust_fuzzer_test_input(&data) };
                let mut lock = RET.lock().unwrap();
                let ret: Option< #args_ret_ty > = lock.take();
                #output_ret
            }
        } else {
            // smoelius: Don't set the panic hook when running under libfuzzer.
            code = test_fuzz::runtime::libfuzzer::libfuzzer_main();
        }

        if code != 0 {
            std::process::exit(code);
        }
    }
}
