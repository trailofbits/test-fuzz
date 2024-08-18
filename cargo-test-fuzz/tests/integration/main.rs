use std::sync::Mutex;

mod auto_generate;
mod breadcrumbs;
mod build;
mod consolidate;
mod display;
mod fuzz;
mod fuzz_cast;
mod fuzz_generic;
mod fuzz_parallel;
mod generic_args;
mod replay;

static ASSERT_MUTEX: Mutex<()> = Mutex::new(());
static PARSE_DURATION_MUTEX: Mutex<()> = Mutex::new(());
