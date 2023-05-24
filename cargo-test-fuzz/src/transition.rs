use super::Object;
use anyhow::Result;
use clap::{crate_version, ArgAction, Parser};
use heck::ToKebabCase;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::{env, ffi::OsStr};

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    TestFuzz(TestFuzzWithDeprecations),
}

// smoelius: Wherever possible, try to reuse cargo test and libtest option names.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Parser, Serialize)]
#[clap(version = crate_version!(), after_help = "To fuzz at most <SECONDS> of time, use:

    cargo test-fuzz ... -- -V <SECONDS>

Try `cargo afl fuzz --help` to see additional fuzzer options.
")]
#[remain::sorted]
struct TestFuzzWithDeprecations {
    #[clap(long, help = "Display backtraces")]
    backtrace: bool,
    #[clap(
        long,
        help = "Move one target's crashes, hangs, and work queue to its corpus; to consolidate \
        all targets, use --consolidate-all"
    )]
    consolidate: bool,
    #[clap(long, hide = true)]
    consolidate_all: bool,
    #[clap(
        long,
        value_name = "OBJECT",
        help = "Display concretizations, corpus, crashes, `impl` concretizations, hangs, or work \
        queue. By default, corpus uses an uninstrumented fuzz target; the others use an \
        instrumented fuzz target. To display the corpus with instrumentation, use --display \
        corpus-instrumented."
    )]
    display: Option<Object>,
    #[clap(long, hide = true)]
    display_concretizations: bool,
    #[clap(long, hide = true)]
    display_corpus: bool,
    #[clap(long, hide = true)]
    display_corpus_instrumented: bool,
    #[clap(long, hide = true)]
    display_crashes: bool,
    #[clap(long, hide = true)]
    display_hangs: bool,
    #[clap(long, hide = true)]
    display_impl_concretizations: bool,
    #[clap(long, hide = true)]
    display_queue: bool,
    #[clap(long, help = "Target name is an exact name rather than a substring")]
    exact: bool,
    #[clap(
        long,
        help = "Exit with 0 if the time limit was reached, 1 for other programmatic aborts, and 2 \
        if an error occurred; implies --no-ui, does not imply --run-until-crash or -- -V <SECONDS>"
    )]
    exit_code: bool,
    #[clap(
        long,
        action = ArgAction::Append,
        help = "Space or comma separated list of features to activate"
    )]
    features: Vec<String>,
    #[clap(long, help = "List fuzz targets")]
    list: bool,
    #[clap(long, value_name = "PATH", help = "Path to Cargo.toml")]
    manifest_path: Option<String>,
    #[clap(long, help = "Do not activate the `default` feature")]
    no_default_features: bool,
    #[clap(
        long,
        help = "Compile without instrumentation (for testing build process)"
    )]
    no_instrumentation: bool,
    #[clap(long, help = "Compile, but don't fuzz")]
    no_run: bool,
    #[clap(long, help = "Disable user interface")]
    no_ui: bool,
    #[clap(short, long, help = "Package containing fuzz target")]
    package: Option<String>,
    #[clap(long, help = "Enable persistent mode fuzzing")]
    persistent: bool,
    #[clap(long, help = "Pretty-print debug output when displaying/replaying")]
    pretty_print: bool,
    #[clap(
        long,
        value_name = "OBJECT",
        help = "Replay corpus, crashes, hangs, or work queue. By default, corpus uses an \
        uninstrumented fuzz target; the others use an instrumented fuzz target. To replay the \
        corpus with instrumentation, use --replay corpus-instrumented."
    )]
    replay: Option<Object>,
    #[clap(long, hide = true)]
    replay_corpus: bool,
    #[clap(long, hide = true)]
    replay_corpus_instrumented: bool,
    #[clap(long, hide = true)]
    replay_crashes: bool,
    #[clap(long, hide = true)]
    replay_hangs: bool,
    #[clap(long, hide = true)]
    replay_queue: bool,
    #[clap(
        long,
        help = "Clear fuzzing data for one target, but leave corpus intact; to reset all \
        targets, use --reset-all"
    )]
    reset: bool,
    #[clap(long, hide = true)]
    reset_all: bool,
    #[clap(long, help = "Resume target's last fuzzing session")]
    resume: bool,
    #[clap(long, help = "Stop fuzzing once a crash is found")]
    run_until_crash: bool,
    #[clap(long, value_name = "TARGETNAME", hide = true)]
    target: Option<String>,
    #[clap(
        long,
        value_name = "NAME",
        help = "Integration test containing fuzz target"
    )]
    test: Option<String>,
    #[clap(
        long,
        help = "Number of seconds to consider a hang when fuzzing or replaying (equivalent \
        to -- -t <TIMEOUT * 1000> when fuzzing)"
    )]
    timeout: Option<u64>,
    #[clap(long, help = "Show build output when displaying/replaying")]
    verbose: bool,
    #[clap(
        value_name = "TARGETNAME",
        help = "String that fuzz target's name must contain"
    )]
    ztarget: Option<String>,
    #[clap(last = true, name = "ARGS", help = "Arguments for the fuzzer")]
    zzargs: Vec<String>,
}

impl From<TestFuzzWithDeprecations> for super::TestFuzz {
    fn from(opts: TestFuzzWithDeprecations) -> Self {
        let TestFuzzWithDeprecations {
            backtrace,
            consolidate,
            consolidate_all,
            display,
            display_concretizations: _,
            display_corpus: _,
            display_corpus_instrumented: _,
            display_crashes: _,
            display_hangs: _,
            display_impl_concretizations: _,
            display_queue: _,
            exact,
            exit_code,
            features,
            list,
            manifest_path,
            no_default_features,
            no_instrumentation,
            no_run,
            no_ui,
            package,
            persistent,
            pretty_print,
            replay,
            replay_corpus: _,
            replay_corpus_instrumented: _,
            replay_crashes: _,
            replay_hangs: _,
            replay_queue: _,
            reset,
            reset_all,
            resume,
            run_until_crash,
            target: _,
            test,
            timeout,
            verbose,
            ztarget,
            zzargs,
        } = opts;
        Self {
            backtrace,
            consolidate,
            consolidate_all,
            display,
            exact,
            exit_code,
            features,
            list,
            manifest_path,
            no_default_features,
            no_instrumentation,
            no_run,
            no_ui,
            package,
            persistent,
            pretty_print,
            replay,
            reset,
            reset_all,
            resume,
            run_until_crash,
            test,
            timeout,
            verbose,
            ztarget,
            zzargs,
        }
    }
}

macro_rules! process_deprecated_action_object {
    ($opts:ident, $action:ident, $object:ident) => {
        paste! {
            if $opts.[< $action _ $object >] {
                eprintln!(
                    "`--{}-{}` is deprecated. Use `--{} {}` (no hyphen).",
                    stringify!($action),
                    stringify!($object).to_kebab_case(),
                    stringify!($action),
                    stringify!($object).to_kebab_case(),
                );
                if $opts.$action.is_none() {
                    $opts.$action = Some(Object::[< $object:camel >]);
                }
            }
        }
    };
}

#[deprecated]
pub fn cargo_test_fuzz<T: AsRef<OsStr>>(args: &[T]) -> Result<()> {
    let SubCommand::TestFuzz(mut opts) = Opts::parse_from(args).subcmd;

    process_deprecated_action_object!(opts, display, corpus);
    process_deprecated_action_object!(opts, display, corpus_instrumented);
    process_deprecated_action_object!(opts, display, crashes);
    process_deprecated_action_object!(opts, display, hangs);
    process_deprecated_action_object!(opts, display, queue);
    process_deprecated_action_object!(opts, display, impl_concretizations);
    process_deprecated_action_object!(opts, display, concretizations);
    process_deprecated_action_object!(opts, replay, corpus);
    process_deprecated_action_object!(opts, replay, corpus_instrumented);
    process_deprecated_action_object!(opts, replay, crashes);
    process_deprecated_action_object!(opts, replay, hangs);
    process_deprecated_action_object!(opts, replay, queue);

    if let Some(target_name) = opts.target.take() {
        eprintln!("`--target <TARGETNAME>` is deprecated. Use just `<TARGETNAME>`.");
        if opts.ztarget.is_none() {
            opts.ztarget = Some(target_name);
        }
    }

    super::run(super::TestFuzz::from(opts))
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Opts::command().debug_assert();
}
