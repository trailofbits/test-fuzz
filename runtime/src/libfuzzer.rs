use libc::{c_char, c_int};
use libfuzzer_sys::fuzz_target;
use nix::{
    sys::wait::{waitpid, WaitStatus},
    unistd::{fork, ForkResult},
};
use serde_json;
use std::{
    ffi::CString,
    sync::atomic::{AtomicPtr, Ordering},
};

extern "C" {
    #[allow(improper_ctypes)]
    fn LLVMFuzzerRunDriver(
        argc: *const c_int,
        argv: *const *const *const c_char,
        callback: fn(data: *const u8, size: usize) -> i32,
    ) -> i32;
}

pub fn libfuzzer_main() -> i32 {
    // Running libfuzzer in a test interferes with its timeout handling. libtest runs each test in
    // its own thread. But libfuzzer expects `SIGALRM` signals to be handled by a specific thread:
    // https://github.com/rust-fuzz/libfuzzer/blob/03a00b2c2ab7b82838536883e0b66eae59ab130d/libfuzzer/FuzzerLoop.cpp#L280
    // To work around this, run libfuzzer in a new child process. Thanks to @maxammann for this very
    // good idea.
    if let ForkResult::Parent { child } = unsafe { fork() }.unwrap() {
        match waitpid(child, None) {
            Ok(WaitStatus::Exited(pid, code)) => {
                assert_eq!(child, pid);
                return code;
            }
            result => panic!("Could not wait for {child}: {result:?}"),
        }
    }

    let args_json =
        std::env::var("TEST_FUZZ_LIBFUZZER_ARGS").expect("`TEST_FUZZ_LIBFUZZER_ARGS` is not set");
    let args = serde_json::from_str::<Vec<CString>>(&args_json).expect(&format!(
        "Could not deserialize `TEST_FUZZ_LIBFUZZER_ARGS`: {:#?}",
        args_json
    ));

    let argv0 = std::env::args()
        .next()
        .and_then(|arg| CString::new(arg).ok())
        .unwrap();
    let argv = std::iter::once(&argv0)
        .chain(args.iter())
        .map(|arg| arg.as_ptr())
        .collect::<Vec<_>>();

    let argc: c_int = argv.len() as c_int;
    let argv: *const *const c_char = &argv[0];

    unsafe { LLVMFuzzerRunDriver(&argc, &argv, libfuzzer_sys::test_input_wrap) }
}

pub static LIBFUZZER_FUZZ_TARGET: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

fuzz_target!(|data: &[u8]| {
    let libfuzzer_fuzz_target_ptr = LIBFUZZER_FUZZ_TARGET.load(Ordering::SeqCst);
    let libfuzzer_fuzz_target =
        unsafe { std::mem::transmute::<_, fn(&[u8])>(libfuzzer_fuzz_target_ptr) };
    libfuzzer_fuzz_target(data);
});
